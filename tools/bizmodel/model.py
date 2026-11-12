"""eBook Cadeau business model.

Pure functions only: nothing here reads the clock, the network or global state.
Every number comes from ``inputs.json`` (low / base / high, unit, source,
confidence); the few structural constants below are documented where they are
defined and listed in ``docs/business/README.md``.

Layers, from the bottom up:

1. ``unit_economics``   what one unit earns in each channel (a waterfall).
2. ``demand``           how many units each month, for an investment level and
                        a scenario (the part with the weakest evidence).
3. ``cash_flow``        revenue, costs, taxes and cash for a list of monthly
                        volumes; ``simulate`` = ``cash_flow(demand(...))``.
4. Metrics              break-even, payback, peak cash need, ROI, self-funding.
5. Uncertainty          ``tornado`` (one input at a time), ``corner_cases``,
                        ``monte_carlo`` (triangular draws, fixed seed).

Conventions: money in EUR; month 1 is January 2027; consumer prices include
VAT; business prices exclude it. "Cash" never includes the founder's unpaid
time; the "valued" figures subtract it at ``founder_hourly_value``.

Stated simplifications (also in the report): corporate tax is paid in the last
month of each year on a cash basis; author plans are recognised monthly; a
reseller's survival is an expected value, not a random event; the personal
income tax of a micro-entrepreneur is not modelled; the micro-entreprise is not
converted into a company when it passes the micro ceiling (a flag is raised).
"""

from __future__ import annotations

import json
import math
import random
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Iterable, Mapping, Sequence

HERE = Path(__file__).resolve().parent
INPUTS_PATH = HERE / "inputs.json"

MONTHS = 36
DEFAULT_SEED = 20261112
DEFAULT_DRAWS = 2000
HOURS_PER_DAY = 7.0
PACK_SIZE = 100

LEVELS = ("minimal", "seed", "funded")
LEVEL_LABELS = {
    "minimal": "Minimal: solo founder, no paid acquisition",
    "seed": "Seed: part-time developer + marketing budget",
    "funded": "Funded: salaried team of 2.5 + marketing",
}

#: Relative weight of January..November in the consumer gift calendar; December
#: takes ``season_december_share`` and the rest is spread with these weights
#: (Valentine's Day, Mother's and Father's Day, a November run-up). Assumption.
SEASON_OTHER_MONTHS = (0.6, 1.1, 0.8, 0.8, 1.2, 1.2, 0.8, 0.7, 0.7, 0.8, 1.6)

#: Order in which resellers are signed: open-file stores first, then storefronts
#: of a bookseller network, then one platform of the Fnac/Kobo class; any
#: further reseller is mid-size (R12 section 9, step 3). Assumption.
PARTNER_LADDER = ("small", "small", "mid", "mid", "large")


@dataclass(frozen=True)
class Scenario:
    """A set of channels switched on together."""

    key: str
    label: str
    channels: frozenset
    pdf_fee: bool = False


SCENARIOS = {
    "A_free": Scenario("A_free", "A. Print-at-home, free", frozenset("a")),
    "A_fee": Scenario("A_fee", "A'. Print-at-home, small fee", frozenset("a"), pdf_fee=True),
    "B_mailed": Scenario("B_mailed", "B. Free generator + printed-and-posted card", frozenset("ab")),
    "C_reseller": Scenario("C_reseller", "C. Reseller fee per card, monthly minimum", frozenset("c")),
    "D_authors": Scenario("D_authors", "D. Author / publisher plan and packs", frozenset("d")),
    "MIX": Scenario("MIX", "Mix: free generator + authors + resellers", frozenset("acd")),
}

#: Channel code -> (scenario that carries its fixed costs, price input).
CHANNEL_PRICE_INPUT = {
    "a": "price_paid_pdf",
    "b": "price_mailed_card",
    "c": "price_reseller_fee_per_card",
    "d": "price_author_jacket",
    "d_plan": "price_author_plan",
}
CHANNEL_SCENARIO = {"a": "A_fee", "b": "B_mailed", "c": "C_reseller", "d": "D_authors", "d_plan": "D_authors"}
CHANNEL_UNIT = {
    "a": "paid PDF",
    "b": "mailed card",
    "c": "card generated at a reseller",
    "d": "jacket in a pack of 100",
    "d_plan": "author plan (one year)",
}


# --------------------------------------------------------------------------- #
# Inputs
# --------------------------------------------------------------------------- #

REQUIRED_FIELDS = ("low", "base", "high", "unit", "role", "vat", "confidence", "source", "section")


def load_inputs(path: Path = INPUTS_PATH) -> dict:
    """Read ``inputs.json`` and check its shape. Returns the ``inputs`` table."""
    with open(path, encoding="utf-8") as handle:
        document = json.load(handle)
    table = document["inputs"]
    documents = document["meta"]["documents"]
    for name, row in table.items():
        missing = [key for key in REQUIRED_FIELDS if key not in row]
        if missing:
            raise ValueError(f"{name}: missing {missing}")
        if not row["low"] <= row["base"] <= row["high"]:
            raise ValueError(f"{name}: low <= base <= high does not hold")
        if row["source"] not in documents:
            raise ValueError(f"{name}: unknown source {row['source']}")
    return table


def load_meta(path: Path = INPUTS_PATH) -> dict:
    """Return the ``meta`` block of ``inputs.json``."""
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)["meta"]


def values(inputs: Mapping, which: str = "base", overrides: Mapping | None = None) -> dict:
    """Flatten the table to ``{name: number}`` at ``low``, ``base`` or ``high``."""
    flat = {name: float(row[which]) for name, row in inputs.items()}
    if overrides:
        flat.update(overrides)
    return flat


# --------------------------------------------------------------------------- #
# Small helpers: seasonality, VAT, payment fees
# --------------------------------------------------------------------------- #


def seasonality_weights(december_share: float) -> tuple:
    """Twelve monthly weights (January first) that sum to 12."""
    if not 0.0 < december_share < 1.0:
        raise ValueError("december_share must be between 0 and 1")
    total = sum(SEASON_OTHER_MONTHS)
    others = [12.0 * (1.0 - december_share) * weight / total for weight in SEASON_OTHER_MONTHS]
    return tuple(others) + (12.0 * december_share,)


def regime_for(level: str) -> str:
    """Tax regime at launch: micro-entreprise under the VAT franchise, or a company."""
    if level not in LEVELS:
        raise ValueError(f"unknown level {level}")
    return "micro_franchise" if level == "minimal" else "company"


def charges_vat(regime: str) -> bool:
    return regime != "micro_franchise"


def is_micro(regime: str) -> bool:
    return regime.startswith("micro")


def net_of_vat(price_incl_vat: float, v: Mapping, regime: str) -> float:
    """What the business keeps of a consumer price."""
    return price_incl_vat / (1.0 + v["tax_vat_standard"]) if charges_vat(regime) else price_incl_vat


def cost_excl_vat(amount_excl_vat: float, v: Mapping, regime: str) -> float:
    """Real cost of a purchase quoted excluding VAT: VAT is lost under the franchise."""
    return amount_excl_vat if charges_vat(regime) else amount_excl_vat * (1.0 + v["tax_vat_standard"])


def card_fee(amount: float, v: Mapping) -> float:
    """Card payment fee on one consumer transaction."""
    return amount * v["pay_card_pct"] + v["pay_card_fixed"] if amount > 0 else 0.0


def sepa_fee(amount: float, v: Mapping) -> float:
    """SEPA direct-debit fee on one business invoice."""
    if amount <= 0:
        return 0.0
    return min(v["pay_sepa_cap"], amount * v["pay_sepa_pct"] + v["pay_sepa_fixed"])


# --------------------------------------------------------------------------- #
# 1. Unit economics
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class UnitEconomics:
    """Waterfall for one unit of one channel.

    ``steps`` starts with the price paid and ends before the contribution; every
    later step is a signed amount (negative = deduction). ``minutes`` is the
    founder or support time the unit consumes; it is in ``steps`` only when
    ``paid_support`` was requested.
    """

    channel: str
    regime: str
    price: float
    steps: tuple
    net_revenue: float
    contribution: float
    minutes: float

    @property
    def variable_cost(self) -> float:
        return self.net_revenue - self.contribution


def unit_economics(
    channel: str,
    v: Mapping,
    regime: str = "micro_franchise",
    year: int = 0,
    price: float | None = None,
    paid_support: bool = False,
) -> UnitEconomics:
    """Contribution of one unit, before fixed costs.

    ``channel``: ``a`` paid print-at-home PDF, ``b`` printed-and-posted card,
    ``c`` card generated at a reseller (metered fee, minimum ignored),
    ``d`` one jacket in an author pack of 100, ``d_plan`` one author-year.
    """
    if channel not in CHANNEL_PRICE_INPUT:
        raise ValueError(f"unknown channel {channel}")
    if price is None:
        price = v[CHANNEL_PRICE_INPUT[channel]]
    business_customer = channel == "c"
    net = price if business_customer else net_of_vat(price, v, regime)
    steps = [("Price paid", price)]
    if net != price:
        steps.append(("VAT 20 %", net - price))
    minutes = 0.0

    if channel == "a":
        steps.append(("Payment fee", -card_fee(price, v)))
        steps.append(("Hosting", -cost_excl_vat(v["var_infra_per_card"], v, regime)))
        social_rate = v["tax_micro_social_services"]
        minutes = v["support_incident_rate"] * v["support_minutes_per_incident"]
    elif channel == "b":
        postage = v["fulfil_postage"] * (1.0 + v["postage_yearly_increase"]) ** year
        steps.append(("Print + envelope", -v["fulfil_card_print"]))
        steps.append(("Postage", -postage))
        steps.append(("Payment fee", -card_fee(price, v)))
        social_rate = v["tax_micro_social_goods"]
        minutes = v["support_incident_rate"] * v["support_minutes_per_incident"]
    elif channel == "c":
        steps.append(("Hosting", -cost_excl_vat(v["var_infra_per_card"], v, regime)))
        social_rate = v["tax_micro_social_services"]
        minutes = v["support_b2b_ticket_rate"] * v["support_minutes_per_incident"]
    elif channel == "d":
        print_cost = v["print_jacket_pack100"] + v["print_variable_data_surcharge"]
        steps.append(("Print (pack of 100)", -cost_excl_vat(print_cost, v, regime)))
        if v["pack_shipping"] > 0:
            steps.append(("Shipping", -v["pack_shipping"] / PACK_SIZE))
        steps.append(("Payment fee", -card_fee(price * PACK_SIZE, v) / PACK_SIZE))
        social_rate = v["tax_micro_social_goods"]
        minutes = v["pack_prepress_minutes"] / PACK_SIZE
    else:  # d_plan
        steps.append(("Payment fee", -card_fee(price, v)))
        social_rate = v["tax_micro_social_services"]

    if is_micro(regime):
        steps.append(("Social charges (micro)", -social_rate * net))
    if paid_support and minutes > 0:
        steps.append(("Support (paid)", -minutes / 60.0 * v["labour_support_hourly_cost"]))
    contribution = price + sum(amount for _, amount in steps[1:])
    return UnitEconomics(channel, regime, price, tuple(steps), net, contribution, minutes)


# --------------------------------------------------------------------------- #
# Fixed costs, people, set-up
# --------------------------------------------------------------------------- #


def fixed_costs_month(v: Mapping, level: str, scenario: str, regime: str, cfe_due: bool) -> dict:
    """Monthly cash fixed costs by line, excluding people and marketing."""
    channels = SCENARIOS[scenario].channels
    lines = {
        "hosting": cost_excl_vat(v["fix_hosting_month"], v, regime),
        "e-mail": v["fix_email_month"],
        "domains": v["fix_domains_year"] / 12.0,
        "insurance": v["fix_rc_pro_year"] / 12.0,
        "bank": v["fix_bank_month"],
        "e-invoicing": v["fix_einvoicing_month"],
    }
    if cfe_due:
        lines["CFE"] = v["fix_cfe_year"] / 12.0
    if "b" in channels or "d" in channels:
        lines["EPR registration"] = v["fix_epr_year"] / 12.0
    if "b" in channels:
        lines["postal API plan"] = v["fix_postal_api_month"]
    if is_micro(regime):
        lines["accounting"] = v["fix_accountant_micro_year"] / 12.0
    else:
        lines["accounting"] = v["fix_accountant_company_year"] / 12.0
    if level == "seed":
        lines["tools"] = v["fix_tools_seed_month"]
    elif level == "funded":
        lines["tools"] = v["fix_tools_funded_month"]
    return lines


def people_costs_month(v: Mapping, level: str) -> float:
    """Monthly cash paid to people (freelance or salaried). Zero at the minimal level."""
    if level == "seed":
        return v["seed_dev_days_month"] * v["labour_dev_day_rate"]
    if level == "funded":
        return v["funded_founder_salary_month"] + v["funded_dev_cost_month"] + v["funded_bizdev_cost_month"]
    return 0.0


def marketing_budget_month(v: Mapping, level: str) -> float:
    if level == "seed":
        return v["seed_marketing_month"]
    if level == "funded":
        return v["funded_marketing_month"]
    return 0.0


def founder_base_hours(v: Mapping, level: str) -> float:
    """Unpaid founder hours a month, before support and handling."""
    if level == "minimal":
        return v["minimal_founder_hours_month"]
    if level == "seed":
        return v["seed_founder_hours_month"]
    return 0.0  # salaried at the funded level


def setup_costs(v: Mapping, level: str, scenario: str) -> dict:
    """One-off cash costs paid in month 1."""
    channels = SCENARIOS[scenario].channels
    lines = {
        "finish the software": v["setup_build_completion"],
        "trade mark (INPI)": v["setup_trademark_inpi"],
    }
    if level == "minimal":
        lines["terms and privacy (templates)"] = v["setup_legal_templates"]
    else:
        lines["terms and privacy (lawyer)"] = v["setup_legal_lawyer"]
        lines["company formation"] = v["setup_company_formation"]
    if level == "funded":
        lines["trade mark (EUIPO)"] = v["setup_trademark_euipo"]
    if "b" in channels or "d" in channels:
        lines["samples and test orders"] = v["setup_samples_test_orders"]
    return lines


# --------------------------------------------------------------------------- #
# 2. Demand
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class MonthVolumes:
    """What happens in one month, before any money is counted."""

    generated: float = 0.0  # cards made with the standalone generator
    paid_pdf: float = 0.0  # of which paid for (small-fee variant)
    mailed: float = 0.0  # printed-and-posted cards
    partner_cards: tuple = ()  # cards at each live reseller, if it is still a customer
    partner_survival: tuple = ()  # probability that each is still a customer
    new_partners: int = 0
    authors: float = 0.0  # paying authors
    packs: float = 0.0  # packs of 100 jackets ordered
    marketing_spend: float = 0.0


def partner_live_months(v: Mapping, level: str) -> list:
    """Simulation month in which each reseller goes live, in signing order."""
    count = max(0, int(round(v[f"{level}_max_partners"])))
    first = v[f"{level}_first_partner_month"]
    gap = v[f"{level}_months_between_partners"]
    return [int(math.ceil(first + index * gap - 1e-9)) for index in range(count)]


def partner_size(index: int) -> str:
    return PARTNER_LADDER[index] if index < len(PARTNER_LADDER) else "mid"


def demand(v: Mapping, level: str, scenario: str, months: int = MONTHS) -> list:
    """Monthly volumes for an investment level and a scenario.

    Marketing money is spent only on channels that can use it (the consumer
    generator, authors) and only up to their caps.
    """
    sc = SCENARIOS[scenario]
    weights = seasonality_weights(v["season_december_share"])
    consumer_on = "a" in sc.channels or "b" in sc.channels
    authors_on = "d" in sc.channels
    resellers_on = "c" in sc.channels

    budget = marketing_budget_month(v, level)
    if consumer_on and authors_on:
        budget_authors = budget * v["mkt_split_authors"]
        budget_consumer = budget - budget_authors
    else:
        budget_consumer = budget if consumer_on else 0.0
        budget_authors = budget if authors_on else 0.0

    consumer_cap = v["b2c_sam_cards_year"] * v["b2c_standalone_cap_share"] / 12.0
    author_churn_month = 1.0 - (1.0 - v["author_annual_churn"]) ** (1.0 / 12.0)
    live_months = partner_live_months(v, level) if resellers_on else []
    authors = 0.0
    out = []

    for month in range(1, months + 1):
        weight = weights[(month - 1) % 12]
        spend = 0.0

        generated = paid_pdf = mailed = 0.0
        if consumer_on:
            organic = min(consumer_cap, v["b2c_organic_cards_month_start"] * (1.0 + v["b2c_organic_growth_month"]) ** (month - 1))
            bought = min(budget_consumer / v["b2c_cac_per_card"], max(0.0, consumer_cap - organic))
            spend += bought * v["b2c_cac_per_card"]
            visitors = (organic + bought) * weight
            if sc.pdf_fee:
                paid_pdf = visitors * v["b2c_paid_pdf_conversion"]
                generated = paid_pdf
            else:
                generated = visitors
            if "b" in sc.channels:
                mailed = generated * v["b2c_mailed_upgrade_share"]

        packs = 0.0
        if authors_on:
            kept = authors * (1.0 - author_churn_month)
            room = max(0.0, v["author_market_cap"] - kept)
            organic_new = min(v[f"{level}_authors_new_month"], room)
            bought_new = min(budget_authors / v["author_cac"], room - organic_new)
            spend += bought_new * v["author_cac"]
            authors = kept + organic_new + bought_new
            packs = authors * v["author_packs_per_year"] / 12.0

        cards, survival, new_partners = [], [], 0
        for index, live in enumerate(live_months):
            if month < live:
                continue
            age = month - live
            if age == 0:
                new_partners += 1
            orders = v[f"reseller_orders_{partner_size(index)}"]
            ramp = min(1.0, (age + 1) / max(1.0, v["reseller_ramp_months"]))
            cards.append(orders / 12.0 * v["reseller_attach_rate"] * ramp * weight)
            survival.append((1.0 - v["reseller_annual_churn"]) ** (age / 12.0))

        out.append(
            MonthVolumes(
                generated=generated,
                paid_pdf=paid_pdf,
                mailed=mailed,
                partner_cards=tuple(cards),
                partner_survival=tuple(survival),
                new_partners=new_partners,
                authors=authors if authors_on else 0.0,
                packs=packs,
                marketing_spend=spend,
            )
        )
    return out


def flat_volumes(
    v: Mapping,
    level: str,
    months: int = MONTHS,
    reseller_cards_year: float = 0.0,
    partners: int = 1,
    authors: float = 0.0,
    mailed_year: float = 0.0,
    paid_pdf_year: float = 0.0,
    generated_year: float = 0.0,
    spend_budget: bool = True,
) -> list:
    """Constant yearly volumes from month 1 (with seasonality), for what-if charts."""
    weights = seasonality_weights(v["season_december_share"])
    partners = max(1, int(partners))
    out = []
    for month in range(1, months + 1):
        weight = weights[(month - 1) % 12]
        per_partner = reseller_cards_year / 12.0 * weight / partners
        out.append(
            MonthVolumes(
                generated=generated_year / 12.0 * weight,
                paid_pdf=paid_pdf_year / 12.0 * weight,
                mailed=mailed_year / 12.0 * weight,
                partner_cards=(per_partner,) * partners if reseller_cards_year > 0 else (),
                partner_survival=(1.0,) * partners if reseller_cards_year > 0 else (),
                authors=authors,
                packs=authors * v["author_packs_per_year"] / 12.0,
                marketing_spend=marketing_budget_month(v, level) if spend_budget else 0.0,
            )
        )
    return out


# --------------------------------------------------------------------------- #
# 3. Cash flow
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class MonthRow:
    month: int
    regime: str
    cards: float  # every card or jacket produced, all channels
    paying_units: float  # mailed cards + paid PDFs + reseller cards + jackets
    partners_live: float
    authors: float
    revenue: float  # excl. VAT
    variable_costs: float  # incl. micro social charges and paid support
    contribution: float
    fixed_costs: float
    people: float
    marketing: float
    setup: float
    tax: float
    net_cash: float
    cum_cash: float
    founder_hours: float
    founder_value: float


@dataclass
class SimResult:
    level: str
    scenario: str
    rows: list
    flags: dict = field(default_factory=dict)

    def series(self, name: str) -> list:
        return [getattr(row, name) for row in self.rows]

    def total(self, name: str, first: int = 1, last: int | None = None) -> float:
        last = last or len(self.rows)
        return sum(getattr(row, name) for row in self.rows[first - 1 : last])


def cash_flow(v: Mapping, level: str, scenario: str, volumes: Sequence) -> SimResult:
    """Turn monthly volumes into revenue, costs, tax and cash."""
    sc = SCENARIOS[scenario]
    regime = regime_for(level)
    paid_support = level != "minimal"
    setup = setup_costs(v, level, scenario)
    flags: dict = {}
    rows = []
    cum = 0.0
    services_history: list = []  # turnover by month, for the VAT franchise test
    total_history: list = []
    year_turnover = 0.0
    previous_year_turnover = 0.0
    year_profit = 0.0
    loss_carried = 0.0
    counsel_paid = False

    for index, vol in enumerate(volumes):
        month = index + 1
        year = index // 12
        services = goods = variable = hours = 0.0

        # a. standalone generator
        if sc.pdf_fee:
            ue = unit_economics("a", v, regime, year, paid_support=paid_support)
            services += vol.paid_pdf * ue.net_revenue
            variable += vol.paid_pdf * ue.variable_cost
            hours += vol.paid_pdf * ue.minutes / 60.0
        else:
            variable += vol.generated * cost_excl_vat(v["var_infra_per_card"], v, regime)

        # b. printed-and-posted card
        if vol.mailed:
            ue = unit_economics("b", v, regime, year, paid_support=paid_support)
            goods += vol.mailed * ue.net_revenue
            variable += vol.mailed * ue.variable_cost
            hours += vol.mailed * ue.minutes / 60.0

        # c. resellers: invoice = max(monthly minimum, cards x fee), per reseller
        partners_live = 0.0
        reseller_cards = 0.0
        if vol.partner_cards:
            ue = unit_economics("c", v, regime, year, paid_support=paid_support)
            per_card_cost = ue.variable_cost - (v["tax_micro_social_services"] * ue.net_revenue if is_micro(regime) else 0.0)
            for cards, alive in zip(vol.partner_cards, vol.partner_survival):
                invoice = max(v["price_reseller_monthly_minimum"], cards * v["price_reseller_fee_per_card"])
                services += alive * invoice
                variable += alive * (cards * per_card_cost + sepa_fee(invoice, v))
                if is_micro(regime):
                    variable += alive * invoice * v["tax_micro_social_services"]
                hours += alive * (cards * ue.minutes / 60.0)
                account_hours = alive * v["partner_account_hours_month"]
                if paid_support:
                    variable += account_hours * v["labour_support_hourly_cost"]
                else:
                    hours += account_hours
                partners_live += alive
                reseller_cards += alive * cards
        if vol.new_partners and level == "minimal":
            hours += vol.new_partners * v["partner_onboarding_days"] * HOURS_PER_DAY

        # d. authors: plan (recognised monthly) and packs of 100 jackets
        jackets = vol.packs * PACK_SIZE
        if vol.authors:
            plan = unit_economics("d_plan", v, regime, year)
            services += vol.authors * plan.net_revenue / 12.0
            variable += vol.authors * plan.variable_cost / 12.0
        if jackets:
            ue = unit_economics("d", v, regime, year, paid_support=paid_support)
            goods += jackets * ue.net_revenue
            variable += jackets * ue.variable_cost
            hours += jackets * ue.minutes / 60.0

        if not paid_support:
            founder_hours = founder_base_hours(v, level) + hours
        else:
            founder_hours = founder_base_hours(v, level)

        revenue = services + goods
        contribution = revenue - variable
        cfe_due = year >= 1 and previous_year_turnover > v["tax_cfe_exempt_turnover"]
        fixed = sum(fixed_costs_month(v, level, scenario, regime, cfe_due).values())
        people = cost_excl_vat(people_costs_month(v, level), v, regime) if level == "seed" else people_costs_month(v, level)
        setup_now = sum(setup.values()) if month == 1 else 0.0
        if vol.new_partners and not counsel_paid:
            setup_now += v["setup_counsel_before_reseller"]
            counsel_paid = True

        pre_tax = contribution - fixed - people - vol.marketing_spend - setup_now
        year_profit += pre_tax
        year_turnover += revenue
        tax = 0.0
        if month % 12 == 0:
            if not is_micro(regime):
                taxable = year_profit + loss_carried
                if taxable > 0:
                    reduced = min(taxable, v["tax_corporate_reduced_ceiling"])
                    tax = reduced * v["tax_corporate_reduced_rate"] + (taxable - reduced) * v["tax_corporate_standard_rate"]
                    loss_carried = 0.0
                else:
                    loss_carried = taxable
            previous_year_turnover = year_turnover
            year_turnover = 0.0
            year_profit = 0.0

        net = pre_tax - tax
        cum += net
        rows.append(
            MonthRow(
                month=month,
                regime=regime,
                cards=vol.generated + vol.mailed + reseller_cards + jackets,
                paying_units=vol.paid_pdf + vol.mailed + reseller_cards + jackets,
                partners_live=partners_live,
                authors=vol.authors,
                revenue=revenue,
                variable_costs=variable,
                contribution=contribution,
                fixed_costs=fixed,
                people=people,
                marketing=vol.marketing_spend,
                setup=setup_now,
                tax=tax,
                net_cash=net,
                cum_cash=cum,
                founder_hours=founder_hours,
                founder_value=founder_hours * v["founder_hourly_value"],
            )
        )

        # Micro-entreprise thresholds, tested on the trailing twelve months.
        services_history.append(services)
        total_history.append(revenue)
        if is_micro(regime):
            services_12 = sum(services_history[-12:])
            total_12 = sum(total_history[-12:])
            if regime == "micro_franchise" and (
                services_12 > v["tax_vat_franchise_services"] or total_12 > v["tax_vat_franchise_total"]
            ):
                regime = "micro_vat"
                flags["vat_franchise_exceeded_month"] = month
            if services_12 > v["tax_micro_ceiling_services"] and "micro_ceiling_exceeded_month" not in flags:
                flags["micro_ceiling_exceeded_month"] = month

    return SimResult(level, scenario, rows, flags)


def simulate(v: Mapping, level: str, scenario: str, months: int = MONTHS) -> SimResult:
    """Demand model followed by the cash-flow engine."""
    return cash_flow(v, level, scenario, demand(v, level, scenario, months))


# --------------------------------------------------------------------------- #
# 4. Metrics
# --------------------------------------------------------------------------- #


def peak_cash_need(cum_cash: Sequence) -> float:
    """Largest cumulative deficit: the cash that must be put in."""
    return max(0.0, -min(cum_cash)) if cum_cash else 0.0


def payback_month(cum_cash: Sequence) -> int | None:
    """First month, at or after the deepest deficit, when cumulative cash is back to zero.

    Returns 0 when cash never goes negative and ``None`` when it does not come
    back within the horizon.
    """
    if not cum_cash or min(cum_cash) >= 0:
        return 0
    trough = min(range(len(cum_cash)), key=lambda i: cum_cash[i])
    for index in range(trough, len(cum_cash)):
        if cum_cash[index] >= 0:
            return index + 1
    return None


def self_funding_month(net_cash: Sequence) -> int | None:
    """First month from which the following twelve months are cash-positive in total."""
    for start in range(0, len(net_cash) - 11):
        if sum(net_cash[start : start + 12]) >= 0:
            return start + 1
    return None


def outlay(result: SimResult, upto: int, valued: bool = False) -> float:
    """Cash committed that does not vary with volume: set-up, fixed, people, marketing."""
    total = sum(result.total(name, 1, upto) for name in ("setup", "fixed_costs", "people", "marketing"))
    if valued:
        total += result.total("founder_value", 1, upto)
    return total


def roi(result: SimResult, upto: int, valued: bool = False) -> float:
    """Return on investment at month ``upto``: net cash generated / cash committed.

    -1.0 means every euro committed is gone; 0 means it just came back.
    """
    committed = outlay(result, upto, valued)
    if committed <= 0:
        return 0.0
    net = result.rows[upto - 1].cum_cash
    if valued:
        net -= result.total("founder_value", 1, upto)
    return net / committed


def summarise(result: SimResult) -> dict:
    """Headline metrics of one simulation."""
    cum = result.series("cum_cash")
    months = len(cum)
    last_year = (months - 11, months)
    out = {
        "level": result.level,
        "scenario": result.scenario,
        "cum_cash_12": cum[11] if months >= 12 else None,
        "cum_cash_24": cum[23] if months >= 24 else None,
        "cum_cash_36": cum[35] if months >= 36 else None,
        "peak_cash_need": peak_cash_need(cum),
        "payback_month": payback_month(cum),
        "self_funding_month": self_funding_month(result.series("net_cash")),
        "roi_12": roi(result, 12) if months >= 12 else None,
        "roi_24": roi(result, 24) if months >= 24 else None,
        "roi_36": roi(result, 36) if months >= 36 else None,
        "roi_36_valued": roi(result, 36, valued=True) if months >= 36 else None,
        "last_year_revenue": result.total("revenue", *last_year),
        "last_year_net_cash": result.total("net_cash", *last_year),
        "last_year_cards": result.total("cards", *last_year),
        "last_year_paying_units": result.total("paying_units", *last_year),
        "partners_end": result.rows[-1].partners_live,
        "authors_end": result.rows[-1].authors,
        "outlay_total": outlay(result, months),
        "founder_hours_total": result.total("founder_hours"),
        "founder_value_total": result.total("founder_value"),
        "cum_cash_end_valued": cum[-1] - result.total("founder_value"),
        "flags": dict(result.flags),
    }
    return out


def annual_fixed_cash(v: Mapping, level: str, scenario: str) -> float:
    """Cash committed in a steady year: fixed lines, people and the full marketing budget."""
    regime = regime_for(level)
    fixed = sum(fixed_costs_month(v, level, scenario, regime, cfe_due=True).values())
    people = cost_excl_vat(people_costs_month(v, level), v, regime) if level == "seed" else people_costs_month(v, level)
    sc = SCENARIOS[scenario]
    marketing = marketing_budget_month(v, level) if sc.channels & frozenset("abd") else 0.0
    return 12.0 * (fixed + people + marketing)


def channel_contribution(v: Mapping, level: str, channel: str, price: float | None = None) -> float:
    """Cash contribution of one unit at an investment level (support paid at seed and funded)."""
    return unit_economics(channel, v, regime_for(level), 0, price, paid_support=level != "minimal").contribution


def annual_profit(v: Mapping, level: str, channel: str, volume: float, price: float | None = None) -> float:
    """Steady-year cash result of selling ``volume`` units of one channel alone."""
    return volume * channel_contribution(v, level, channel, price) - annual_fixed_cash(v, level, CHANNEL_SCENARIO[channel])


def break_even_volume(v: Mapping, level: str, channel: str, price: float | None = None) -> float:
    """Units a year at which one channel alone covers the level's steady-year cash costs.

    ``math.inf`` when a unit does not earn anything.
    """
    contribution = channel_contribution(v, level, channel, price)
    if contribution <= 0:
        return math.inf
    return annual_fixed_cash(v, level, CHANNEL_SCENARIO[channel]) / contribution


def steady_state_net(v: Mapping, level: str, reseller_cards_year: float, authors: float, scenario: str = "MIX") -> float:
    """Yearly net cash at a given reseller volume and author base (pure metered fee)."""
    regime = regime_for(level)
    paid = level != "minimal"
    fee = unit_economics("c", v, regime, 0, paid_support=paid).contribution
    plan = unit_economics("d_plan", v, regime, 0).contribution
    jacket = unit_economics("d", v, regime, 0, paid_support=paid).contribution
    per_author = plan + v["author_packs_per_year"] * PACK_SIZE * jacket
    return reseller_cards_year * fee + authors * per_author - annual_fixed_cash(v, level, scenario)


def partner_year_income(v: Mapping, orders_year: float, attach_rate: float, regime: str = "company") -> float:
    """What one live reseller brings in a year, after the monthly minimum, fees and charges."""
    weights = seasonality_weights(v["season_december_share"])
    total = 0.0
    for weight in weights:
        cards = orders_year / 12.0 * attach_rate * weight
        invoice = max(v["price_reseller_monthly_minimum"], cards * v["price_reseller_fee_per_card"])
        total += invoice - sepa_fee(invoice, v) - cards * cost_excl_vat(v["var_infra_per_card"], v, regime)
        if is_micro(regime):
            total -= invoice * v["tax_micro_social_services"]
    return total


def partners_needed(v: Mapping, target_year: float, orders_year: float, attach_rate: float, regime: str = "company") -> float:
    """Number of resellers of one size needed to bring in ``target_year`` (not rounded)."""
    income = partner_year_income(v, orders_year, attach_rate, regime)
    return target_year / income if income > 0 else math.inf


# --------------------------------------------------------------------------- #
# 5. Uncertainty
# --------------------------------------------------------------------------- #


def cum_cash_end(result: SimResult) -> float:
    return result.rows[-1].cum_cash


def tornado(
    inputs: Mapping,
    level: str,
    scenario: str,
    metric: Callable = cum_cash_end,
    months: int = MONTHS,
) -> list:
    """One-at-a-time sensitivity: each input at its low and high value, others at base.

    Returns rows sorted by the size of the swing, largest first.
    """
    base = values(inputs)
    base_value = metric(simulate(base, level, scenario, months))
    rows = []
    for name, row in inputs.items():
        if row["low"] == row["high"]:
            continue
        at_low = metric(simulate(values(inputs, overrides={name: row["low"]}), level, scenario, months))
        at_high = metric(simulate(values(inputs, overrides={name: row["high"]}), level, scenario, months))
        swing = abs(at_high - at_low)
        if swing <= 1e-9 and abs(at_low - base_value) <= 1e-9:
            continue
        rows.append(
            {
                "input": name,
                "role": row["role"],
                "low": row["low"],
                "high": row["high"],
                "base": row["base"],
                "unit": row["unit"],
                "value_at_low": at_low,
                "value_at_high": at_high,
                "base_value": base_value,
                "swing": swing,
            }
        )
    rows.sort(key=lambda r: (-r["swing"], r["input"]))
    return rows


def corner_cases(inputs: Mapping, level: str, scenario: str, months: int = MONTHS) -> dict:
    """Every uncertain input at its adverse end, then at its favourable end.

    Decisions (prices, budgets) stay at base. These corners are far less likely
    than the Monte Carlo band; they bound it.
    """
    adverse, favourable = {}, {}
    for row in tornado(inputs, level, scenario, months=months):
        if row["role"] == "decision":
            continue
        low_is_worse = row["value_at_low"] <= row["value_at_high"]
        adverse[row["input"]] = row["low"] if low_is_worse else row["high"]
        favourable[row["input"]] = row["high"] if low_is_worse else row["low"]
    return {
        "adverse": simulate(values(inputs, overrides=adverse), level, scenario, months),
        "favourable": simulate(values(inputs, overrides=favourable), level, scenario, months),
    }


def percentile(sorted_values: Sequence, q: float) -> float:
    """Linear-interpolation percentile of an already sorted sequence (q in 0..100)."""
    if not sorted_values:
        raise ValueError("empty sequence")
    position = (len(sorted_values) - 1) * q / 100.0
    lower = int(math.floor(position))
    upper = min(lower + 1, len(sorted_values) - 1)
    return sorted_values[lower] + (sorted_values[upper] - sorted_values[lower]) * (position - lower)


def draw(inputs: Mapping, rng: random.Random) -> dict:
    """One Monte Carlo draw: triangular on every uncertain input, decisions at base."""
    flat = {}
    for name, row in inputs.items():
        if row["role"] == "uncertain" and row["low"] < row["high"]:
            flat[name] = rng.triangular(row["low"], row["high"], row["base"])
        else:
            flat[name] = float(row["base"])
    return flat


def monte_carlo(
    inputs: Mapping,
    level: str,
    scenario: str,
    draws: int = DEFAULT_DRAWS,
    seed: int = DEFAULT_SEED,
    months: int = MONTHS,
    quantiles: Iterable = (10, 50, 90),
) -> dict:
    """Independent triangular draws over the low / base / high ranges.

    Inputs are drawn independently, which understates the tails when errors are
    correlated (for example every demand assumption being too high at once).
    """
    rng = random.Random(seed)
    quantiles = tuple(quantiles)
    paths, metrics = [], []
    for _ in range(draws):
        result = simulate(draw(inputs, rng), level, scenario, months)
        paths.append(result.series("cum_cash"))
        metrics.append(summarise(result))

    bands = {q: [] for q in quantiles}
    for month in range(months):
        column = sorted(path[month] for path in paths)
        for q in quantiles:
            bands[q].append(percentile(column, q))

    def spread(key: str) -> dict:
        column = sorted(m[key] for m in metrics)
        return {f"p{q}": percentile(column, q) for q in quantiles}

    paid_back = [m["payback_month"] for m in metrics if m["payback_month"] is not None]
    return {
        "level": level,
        "scenario": scenario,
        "draws": draws,
        "seed": seed,
        "cum_cash": bands,
        "cum_cash_end": spread(f"cum_cash_{months}") if months in (12, 24, 36) else None,
        "peak_cash_need": spread("peak_cash_need"),
        "last_year_net_cash": spread("last_year_net_cash"),
        "last_year_revenue": spread("last_year_revenue"),
        "prob_payback": len(paid_back) / draws,
        "prob_positive_end": sum(1 for path in paths if path[-1] >= 0) / draws,
        "prob_last_year_positive": sum(1 for m in metrics if m["last_year_net_cash"] >= 0) / draws,
        "median_payback_month": percentile(sorted(paid_back), 50) if paid_back else None,
    }
