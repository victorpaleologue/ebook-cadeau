"""Invariants of the business model.

    python -m unittest discover -s tools/bizmodel -v

No network, no clock, no matplotlib: only ``model.py`` and ``inputs.json``.
"""

from __future__ import annotations

import math
import random
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import model  # noqa: E402

INPUTS = model.load_inputs()
BASE = model.values(INPUTS)
CHANNELS = ("a", "b", "c", "d", "d_plan")
REGIMES = ("micro_franchise", "micro_vat", "company")


class InputsTest(unittest.TestCase):
    def test_every_input_is_documented_and_ordered(self):
        meta = model.load_meta()
        for name, row in INPUTS.items():
            with self.subTest(name=name):
                for key in model.REQUIRED_FIELDS:
                    self.assertIn(key, row)
                self.assertLessEqual(row["low"], row["base"])
                self.assertLessEqual(row["base"], row["high"])
                self.assertIn(row["source"], meta["documents"])
                self.assertIn(row["role"], ("decision", "uncertain", "fixed"))
                self.assertIn(row["vat"], ("HT", "TTC", "none"))
                self.assertIn(row["confidence"], ("V2", "V1", "M", "derived", "assumption"))
                self.assertTrue(row["unit"])
                if row["role"] == "fixed":
                    self.assertEqual(row["low"], row["high"])

    def test_fact_check_corrections_are_applied(self):
        # FC12 finding 1: landed cost of a mailed card is 4.29 / about 5.3 / 6.33 or more.
        landed = [INPUTS["fulfil_card_print"][k] + INPUTS["fulfil_postage"][k] for k in ("low", "base", "high")]
        self.assertAlmostEqual(landed[0], 4.29, places=2)
        self.assertAlmostEqual(landed[1], 5.30, places=2)
        self.assertGreaterEqual(landed[2], 6.33)
        # FC12 finding 5: the December peak must not be pinned at 45-55 %.
        self.assertLessEqual(INPUTS["season_december_share"]["low"], 0.21)
        self.assertLess(INPUTS["season_december_share"]["base"], 0.45)

    def test_values_overrides(self):
        flat = model.values(INPUTS, "high", {"price_mailed_card": 1.0})
        self.assertEqual(flat["price_mailed_card"], 1.0)
        self.assertEqual(flat["reseller_attach_rate"], INPUTS["reseller_attach_rate"]["high"])


class SeasonalityTest(unittest.TestCase):
    def test_weights_sum_to_twelve_and_december_share_holds(self):
        for share in (0.17, 0.30, 0.50):
            weights = model.seasonality_weights(share)
            self.assertEqual(len(weights), 12)
            self.assertAlmostEqual(sum(weights), 12.0)
            self.assertAlmostEqual(weights[11] / 12.0, share)
            self.assertTrue(all(w > 0 for w in weights))

    def test_rejects_nonsense(self):
        for share in (0.0, 1.0, -0.2, 1.5):
            with self.assertRaises(ValueError):
                model.seasonality_weights(share)


class UnitEconomicsTest(unittest.TestCase):
    def test_contribution_is_price_minus_steps(self):
        for channel in CHANNELS:
            for regime in REGIMES:
                ue = model.unit_economics(channel, BASE, regime)
                self.assertAlmostEqual(ue.contribution, sum(amount for _, amount in ue.steps))
                self.assertAlmostEqual(ue.variable_cost, ue.net_revenue - ue.contribution)
                self.assertTrue(all(amount <= 0 for _, amount in ue.steps[1:]), "every step after the price is a deduction")

    def test_contribution_rises_with_price(self):
        for channel in CHANNELS:
            for regime in REGIMES:
                base_price = BASE[model.CHANNEL_PRICE_INPUT[channel]]
                previous = -math.inf
                for factor in (0.5, 0.8, 1.0, 1.3, 2.0, 5.0):
                    value = model.unit_economics(channel, BASE, regime, price=base_price * factor).contribution
                    self.assertGreater(value, previous, f"{channel} {regime}")
                    previous = value

    def test_contribution_never_exceeds_price(self):
        for which in ("low", "base", "high"):
            flat = model.values(INPUTS, which)
            for channel in CHANNELS:
                for regime in REGIMES:
                    ue = model.unit_economics(channel, flat, regime)
                    self.assertLess(ue.contribution, ue.price)

    def test_vat_and_social_charges_by_regime(self):
        consumer = model.unit_economics("b", BASE, "company")
        self.assertAlmostEqual(consumer.net_revenue, consumer.price / 1.2)
        business = model.unit_economics("c", BASE, "company")
        self.assertAlmostEqual(business.net_revenue, business.price, msg="a business fee is quoted excluding VAT")
        micro = model.unit_economics("b", BASE, "micro_franchise")
        self.assertAlmostEqual(micro.net_revenue, micro.price, msg="no VAT under the franchise")
        self.assertIn("Social charges (micro)", dict(micro.steps))
        self.assertNotIn("Social charges (micro)", dict(consumer.steps))

    def test_input_vat_is_a_cost_under_the_franchise(self):
        franchise = dict(model.unit_economics("d", BASE, "micro_franchise").steps)["Print (pack of 100)"]
        company = dict(model.unit_economics("d", BASE, "company").steps)["Print (pack of 100)"]
        self.assertAlmostEqual(franchise, company * 1.2)

    def test_postage_inflates_with_years(self):
        year0 = model.unit_economics("b", BASE, year=0).contribution
        year2 = model.unit_economics("b", BASE, year=2).contribution
        self.assertLess(year2, year0)

    def test_paid_support_only_when_asked(self):
        free = model.unit_economics("b", BASE, "company")
        paid = model.unit_economics("b", BASE, "company", paid_support=True)
        cost = free.minutes / 60.0 * BASE["labour_support_hourly_cost"]
        self.assertAlmostEqual(free.contribution - paid.contribution, cost)

    def test_mailed_card_matches_the_fact_check_table(self):
        # FC12 section 2: micro-entreprise, landed 4.29 -> +1.41 at 6.90; landed 6.33 -> -0.63.
        low = dict(BASE, fulfil_card_print=2.99, fulfil_postage=1.30)
        self.assertAlmostEqual(model.unit_economics("b", low, price=6.90).contribution, 1.41, places=1)
        high = dict(BASE, fulfil_card_print=2.99, fulfil_postage=3.34)
        self.assertAlmostEqual(model.unit_economics("b", high, price=6.90).contribution, -0.63, places=1)

    def test_unknown_channel(self):
        with self.assertRaises(ValueError):
            model.unit_economics("z", BASE)


class BreakEvenTest(unittest.TestCase):
    def test_profit_is_zero_at_break_even(self):
        for level in model.LEVELS:
            for channel in ("a", "b", "c", "d"):
                for price_factor in (1.0, 1.5, 3.0):
                    price = BASE[model.CHANNEL_PRICE_INPUT[channel]] * price_factor
                    volume = model.break_even_volume(BASE, level, channel, price)
                    if math.isinf(volume):
                        self.assertLessEqual(model.channel_contribution(BASE, level, channel, price), 0)
                        continue
                    fixed = model.annual_fixed_cash(BASE, level, model.CHANNEL_SCENARIO[channel])
                    self.assertAlmostEqual(model.annual_profit(BASE, level, channel, volume, price) / fixed, 0.0, places=9)
                    self.assertGreater(model.annual_profit(BASE, level, channel, volume * 1.01, price), 0)
                    self.assertLess(model.annual_profit(BASE, level, channel, volume * 0.99, price), 0)

    def test_break_even_falls_as_price_rises(self):
        for level in model.LEVELS:
            for channel in ("a", "b", "c", "d"):
                base_price = BASE[model.CHANNEL_PRICE_INPUT[channel]]
                previous = math.inf
                for factor in (0.6, 0.8, 1.0, 1.2, 1.6, 2.5, 4.0):
                    volume = model.break_even_volume(BASE, level, channel, base_price * factor)
                    self.assertLessEqual(volume, previous)
                    if not math.isinf(previous):
                        self.assertLess(volume, previous)
                    previous = volume

    def test_break_even_rises_with_investment_level(self):
        for channel in ("a", "c", "d"):
            volumes = [model.break_even_volume(BASE, level, channel) for level in model.LEVELS]
            self.assertLess(volumes[0], volumes[1])
            self.assertLess(volumes[1], volumes[2])

    def test_no_break_even_when_a_unit_loses_money(self):
        self.assertTrue(math.isinf(model.break_even_volume(BASE, "minimal", "b", price=4.90)))

    def test_engine_agrees_with_unit_economics(self):
        """A flat year in the cash-flow engine = volume x contribution - fixed costs - set-up."""
        volume = 5000.0
        ue = model.unit_economics("b", BASE, "micro_franchise")
        flows = model.cash_flow(BASE, "minimal", "B_mailed", model.flat_volumes(BASE, "minimal", months=12, mailed_year=volume))
        fixed = 12 * sum(model.fixed_costs_month(BASE, "minimal", "B_mailed", "micro_franchise", cfe_due=False).values())
        setup = sum(model.setup_costs(BASE, "minimal", "B_mailed").values())
        self.assertAlmostEqual(flows.rows[-1].cum_cash, volume * ue.contribution - fixed - setup, places=6)
        self.assertAlmostEqual(flows.total("revenue"), volume * ue.net_revenue, places=6)

    def test_steady_state_net_is_linear_below_the_vat_threshold(self):
        zero = model.steady_state_net(BASE, "minimal", 0, 0)
        self.assertAlmostEqual(zero, -model.annual_fixed_cash(BASE, "minimal", "MIX"))
        one = model.steady_state_net(BASE, "minimal", 10000, 0)
        two = model.steady_state_net(BASE, "minimal", 20000, 0)
        self.assertAlmostEqual(two - one, one - zero)
        self.assertGreater(model.steady_state_net(BASE, "minimal", 10000, 50), one)


class SimulationTest(unittest.TestCase):
    def test_deterministic(self):
        first = model.simulate(BASE, "seed", "MIX").series("cum_cash")
        second = model.simulate(dict(BASE), "seed", "MIX").series("cum_cash")
        self.assertEqual(first, second)

    def test_cash_identity_every_month(self):
        for level in model.LEVELS:
            for scenario in model.SCENARIOS:
                result = model.simulate(BASE, level, scenario)
                self.assertEqual(len(result.rows), model.MONTHS)
                cumulative = 0.0
                for row in result.rows:
                    net = row.revenue - row.variable_costs - row.fixed_costs - row.people - row.marketing - row.setup - row.tax
                    self.assertAlmostEqual(row.net_cash, net, places=6)
                    self.assertAlmostEqual(row.contribution, row.revenue - row.variable_costs, places=6)
                    cumulative += row.net_cash
                    self.assertAlmostEqual(row.cum_cash, cumulative, places=6)
                    self.assertGreaterEqual(row.revenue, 0)
                    self.assertGreaterEqual(row.tax, 0)
                    self.assertGreaterEqual(row.cards, 0)

    def test_no_demand_means_only_costs(self):
        empty = [model.MonthVolumes() for _ in range(model.MONTHS)]
        for level in model.LEVELS:
            result = model.cash_flow(BASE, level, "C_reseller", empty)
            self.assertEqual(result.total("revenue"), 0.0)
            self.assertAlmostEqual(result.rows[-1].cum_cash, -model.outlay(result, model.MONTHS))
            self.assertAlmostEqual(model.roi(result, model.MONTHS), -1.0)
            self.assertIsNone(model.payback_month(result.series("cum_cash")))

    def test_more_demand_never_hurts_where_a_unit_earns(self):
        # Metered fee only: under a monthly minimum extra cards are pure cost (see the next test).
        metered = dict(BASE, price_reseller_monthly_minimum=0.0)
        cases = [
            ("C_reseller", "reseller_attach_rate"),
            ("C_reseller", "reseller_orders_small"),
            ("D_authors", "author_packs_per_year"),
            ("D_authors", "minimal_authors_new_month"),
            ("MIX", "reseller_attach_rate"),
        ]
        for scenario, name in cases:
            row = INPUTS[name]
            previous = -math.inf
            for value in (row["low"], row["base"], row["high"], row["high"] * 2):
                end = model.simulate(dict(metered, **{name: value}), "minimal", scenario).rows[-1].cum_cash
                self.assertGreaterEqual(end, previous, f"{scenario} {name}")
                previous = end

    def test_under_the_minimum_extra_cards_only_cost(self):
        """A reseller that pays the minimum either way brings no more money with more cards."""
        low = model.partner_year_income(BASE, BASE["reseller_orders_small"], 0.001)
        more = model.partner_year_income(BASE, BASE["reseller_orders_small"], 0.004)
        self.assertLess(more, low)
        self.assertLess(low - more, 5.0, "the loss is only the hosting of the extra cards")

    def test_higher_prices_never_hurt_at_fixed_volume(self):
        for name, scenario in (("price_reseller_fee_per_card", "C_reseller"), ("price_reseller_monthly_minimum", "C_reseller"),
                               ("price_author_plan", "D_authors"), ("price_author_jacket", "D_authors"),
                               ("price_mailed_card", "B_mailed"), ("price_paid_pdf", "A_fee")):
            row = INPUTS[name]
            for level in model.LEVELS:
                ends = [model.simulate(dict(BASE, **{name: value}), level, scenario).rows[-1].cum_cash
                        for value in (row["low"], row["base"], row["high"])]
                self.assertLessEqual(ends[0], ends[1], f"{name} {level}")
                self.assertLessEqual(ends[1], ends[2], f"{name} {level}")

    def test_flat_volume_cash_rises_with_volume(self):
        previous = -math.inf
        for cards in (0, 1000, 10000, 100000, 1000000):
            flows = model.cash_flow(BASE, "seed", "C_reseller", model.flat_volumes(BASE, "seed", reseller_cards_year=cards))
            self.assertGreater(flows.rows[-1].cum_cash, previous)
            previous = flows.rows[-1].cum_cash

    def test_free_generator_earns_nothing_and_buys_no_advertising(self):
        for level in model.LEVELS:
            result = model.simulate(BASE, level, "A_free")
            self.assertEqual(result.total("revenue"), 0.0)
            self.assertEqual(result.total("marketing"), 0.0)
            self.assertLess(result.rows[-1].cum_cash, 0)

    def test_marketing_never_exceeds_the_budget(self):
        for level in model.LEVELS:
            budget = model.marketing_budget_month(BASE, level)
            for scenario in model.SCENARIOS:
                for row in model.simulate(BASE, level, scenario).rows:
                    self.assertLessEqual(row.marketing, budget + 1e-9)

    def test_caps_hold(self):
        result = model.simulate(model.values(INPUTS, "high"), "funded", "MIX")
        self.assertLessEqual(max(result.series("authors")), INPUTS["author_market_cap"]["high"] + 1e-9)
        self.assertLessEqual(result.rows[-1].partners_live, INPUTS["funded_max_partners"]["high"])

    def test_reseller_invoice_respects_the_minimum(self):
        volumes = model.demand(BASE, "minimal", "C_reseller")
        result = model.cash_flow(BASE, "minimal", "C_reseller", volumes)
        for vol, row in zip(volumes, result.rows):
            floor = sum(vol.partner_survival) * BASE["price_reseller_monthly_minimum"]
            self.assertGreaterEqual(row.revenue + 1e-9, floor)

    def test_vat_franchise_switch(self):
        big = model.flat_volumes(BASE, "minimal", reseller_cards_year=400000)
        result = model.cash_flow(BASE, "minimal", "C_reseller", big)
        self.assertIn("vat_franchise_exceeded_month", result.flags)
        self.assertIn("micro_ceiling_exceeded_month", result.flags)
        switched = result.flags["vat_franchise_exceeded_month"]
        self.assertEqual(result.rows[switched - 1].regime, "micro_franchise")
        self.assertEqual(result.rows[switched].regime, "micro_vat")
        small = model.cash_flow(BASE, "minimal", "C_reseller", model.flat_volumes(BASE, "minimal", reseller_cards_year=5000))
        self.assertEqual(small.flags, {})

    def test_corporate_tax_only_on_profit_after_losses(self):
        loss = model.simulate(BASE, "funded", "MIX")
        self.assertEqual(loss.total("tax"), 0.0)
        rich = model.cash_flow(BASE, "seed", "C_reseller", model.flat_volumes(BASE, "seed", reseller_cards_year=2_000_000))
        self.assertGreater(rich.total("tax"), 0.0)
        self.assertTrue(all(row.tax == 0 for row in rich.rows if row.month % 12 != 0))
        self.assertTrue(all(row.tax == 0 for row in model.simulate(BASE, "minimal", "MIX").rows), "no corporate tax in a micro-entreprise")

    def test_founder_time_is_never_cash(self):
        cheap = model.simulate(dict(BASE, founder_hourly_value=0.0), "minimal", "MIX")
        dear = model.simulate(dict(BASE, founder_hourly_value=500.0), "minimal", "MIX")
        self.assertEqual(cheap.series("cum_cash"), dear.series("cum_cash"))
        self.assertGreater(dear.total("founder_value"), cheap.total("founder_value"))
        self.assertEqual(model.simulate(BASE, "funded", "MIX").total("founder_hours"), 0.0)


class MetricsTest(unittest.TestCase):
    def test_payback_month(self):
        self.assertEqual(model.payback_month([1, 2, 3]), 0)
        self.assertEqual(model.payback_month([-5, -8, -2, 1, 4]), 4)
        self.assertIsNone(model.payback_month([-5, -8, -2]))
        # A dip after an early positive month: payback is counted after the deepest point.
        self.assertEqual(model.payback_month([1, -3, -9, -1, 0, 2]), 5)

    def test_payback_consistent_with_the_path(self):
        for level in model.LEVELS:
            for scenario in model.SCENARIOS:
                cum = model.simulate(BASE, level, scenario).series("cum_cash")
                month = model.payback_month(cum)
                if month:
                    self.assertGreaterEqual(cum[month - 1], 0)
                    self.assertLess(min(cum[: month - 1]), 0)
                elif month is None:
                    self.assertLess(cum[-1], 0)

    def test_peak_cash_need(self):
        self.assertEqual(model.peak_cash_need([5, 3, 9]), 0.0)
        self.assertEqual(model.peak_cash_need([-5, -12, 4]), 12)

    def test_self_funding_month(self):
        self.assertEqual(model.self_funding_month([1.0] * 12), 1)
        self.assertIsNone(model.self_funding_month([1.0] * 11))
        self.assertEqual(model.self_funding_month([-200.0] * 3 + [10.0] * 12), 4)
        self.assertIsNone(model.self_funding_month([-1.0] * 36))

    def test_roi_definition(self):
        result = model.simulate(BASE, "minimal", "MIX")
        committed = model.outlay(result, 36)
        self.assertAlmostEqual(model.roi(result, 36), result.rows[35].cum_cash / committed)
        self.assertLess(model.roi(result, 36, valued=True), model.roi(result, 36))
        summary = model.summarise(result)
        self.assertAlmostEqual(summary["roi_36"], model.roi(result, 36))
        self.assertAlmostEqual(summary["cum_cash_36"], result.rows[-1].cum_cash)
        self.assertAlmostEqual(summary["peak_cash_need"], model.peak_cash_need(result.series("cum_cash")))

    def test_partners_needed(self):
        target = 63300.0
        metered = dict(BASE, price_reseller_monthly_minimum=0.0)
        for size in ("small", "mid", "large"):
            orders = BASE[f"reseller_orders_{size}"]
            previous = math.inf
            for rate in (0.002, 0.005, 0.02, 0.05, 0.10):
                needed = model.partners_needed(metered, target, orders, rate)
                self.assertLess(needed, previous, "with a pure metered fee, a higher attach rate needs fewer resellers")
                previous = needed
                income = model.partner_year_income(BASE, orders, rate)
                self.assertAlmostEqual(model.partners_needed(BASE, target, orders, rate) * income, target)
                self.assertGreaterEqual(income, model.partner_year_income(metered, orders, rate), "a minimum never lowers income")
        self.assertLess(model.partners_needed(BASE, target, BASE["reseller_orders_large"], 0.02),
                        model.partners_needed(BASE, target, BASE["reseller_orders_small"], 0.02))


class UncertaintyTest(unittest.TestCase):
    def test_tornado_base_and_order(self):
        rows = model.tornado(INPUTS, "minimal", "MIX")
        base = model.simulate(BASE, "minimal", "MIX").rows[-1].cum_cash
        self.assertTrue(rows)
        swings = [row["swing"] for row in rows]
        self.assertEqual(swings, sorted(swings, reverse=True))
        for row in rows:
            self.assertAlmostEqual(row["base_value"], base)
            self.assertAlmostEqual(row["swing"], abs(row["value_at_high"] - row["value_at_low"]))
        by_name = {row["input"]: row for row in rows}
        self.assertGreater(by_name["reseller_attach_rate"]["value_at_high"], by_name["reseller_attach_rate"]["value_at_low"])
        self.assertLess(by_name["print_jacket_pack100"]["value_at_high"], by_name["print_jacket_pack100"]["value_at_low"])
        self.assertNotIn("price_mailed_card", by_name, "inputs of channels that are off must not appear")

    def test_corners_bracket_the_base(self):
        corners = model.corner_cases(INPUTS, "minimal", "MIX")
        base = model.simulate(BASE, "minimal", "MIX").rows[-1].cum_cash
        self.assertLess(corners["adverse"].rows[-1].cum_cash, base)
        self.assertGreater(corners["favourable"].rows[-1].cum_cash, base)

    def test_percentile(self):
        data = [1.0, 2.0, 3.0, 4.0, 5.0]
        self.assertEqual(model.percentile(data, 0), 1.0)
        self.assertEqual(model.percentile(data, 50), 3.0)
        self.assertEqual(model.percentile(data, 100), 5.0)
        self.assertAlmostEqual(model.percentile(data, 10), 1.4)
        with self.assertRaises(ValueError):
            model.percentile([], 50)

    def test_draw_stays_in_range_and_keeps_decisions(self):
        rng = random.Random(1)
        for _ in range(200):
            flat = model.draw(INPUTS, rng)
            for name, row in INPUTS.items():
                self.assertGreaterEqual(flat[name], row["low"])
                self.assertLessEqual(flat[name], row["high"])
                if row["role"] != "uncertain":
                    self.assertEqual(flat[name], row["base"])

    def test_monte_carlo_is_reproducible(self):
        first = model.monte_carlo(INPUTS, "minimal", "MIX", draws=150, seed=7)
        again = model.monte_carlo(INPUTS, "minimal", "MIX", draws=150, seed=7)
        other = model.monte_carlo(INPUTS, "minimal", "MIX", draws=150, seed=8)
        self.assertEqual(first, again)
        self.assertNotEqual(first["cum_cash"][50], other["cum_cash"][50])

    def test_monte_carlo_bands_are_ordered_and_bracket_the_base(self):
        mc = model.monte_carlo(INPUTS, "minimal", "MIX", draws=300)
        for low, mid, high in zip(mc["cum_cash"][10], mc["cum_cash"][50], mc["cum_cash"][90]):
            self.assertLessEqual(low, mid)
            self.assertLessEqual(mid, high)
        corners = model.corner_cases(INPUTS, "minimal", "MIX")
        self.assertGreater(mc["cum_cash"][10][-1], corners["adverse"].rows[-1].cum_cash)
        self.assertLess(mc["cum_cash"][90][-1], corners["favourable"].rows[-1].cum_cash)
        self.assertGreaterEqual(mc["prob_payback"], 0.0)
        self.assertLessEqual(mc["prob_payback"], 1.0)
        self.assertLessEqual(mc["peak_cash_need"]["p10"], mc["peak_cash_need"]["p90"])


class HostileInputTest(unittest.TestCase):
    def test_extreme_values_do_not_crash(self):
        hostile = dict(BASE, reseller_attach_rate=0.0, b2c_organic_cards_month_start=0.0, minimal_authors_new_month=0.0,
                       minimal_max_partners=0.0, author_market_cap=0.0, reseller_ramp_months=0.0)
        for scenario in model.SCENARIOS:
            result = model.simulate(hostile, "minimal", scenario)
            self.assertTrue(all(math.isfinite(row.cum_cash) for row in result.rows))
        self.assertTrue(math.isinf(model.partners_needed(dict(BASE, price_reseller_monthly_minimum=0.0), 1000.0, 1e5, 0.0)))

    def test_unknown_level(self):
        with self.assertRaises(ValueError):
            model.regime_for("galactic")


if __name__ == "__main__":
    unittest.main()
