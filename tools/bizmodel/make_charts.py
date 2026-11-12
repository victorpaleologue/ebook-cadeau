"""Run the eBook Cadeau business model and draw its charts.

    python tools/bizmodel/make_charts.py

Writes, under ``docs/business/``:

* ``results.json``             every number quoted in the report;
* ``charts/NN-name.svg|png``   light theme, and ``NN-name-dark.svg|png``;
* ``charts/data/NN-name.csv``  the table behind each chart.

Output is deterministic: fixed Monte Carlo seed, no timestamps in the files,
text converted to outlines in the SVGs so they render the same everywhere.

Chart rules followed (the ``dataviz`` skill): one question per chart; colour
does one job per chart (blue = the measured quantity, grey = context, red =
below zero or worse than base, a one-hue ramp for ordered series); both themes
use validated steps of the same palette; thin marks, solid hairline grids,
values labelled selectively, text never in a data colour, a CSV twin for every
chart.
"""

from __future__ import annotations

import csv
import json
import math
import sys
import textwrap
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib import font_manager  # noqa: E402
from matplotlib.patches import PathPatch  # noqa: E402
from matplotlib.path import Path as MplPath  # noqa: E402
from matplotlib.ticker import FixedLocator, FuncFormatter, NullLocator  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parent))
import model  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "business"
CHARTS = OUT / "charts"
DATA = CHARTS / "data"

PNG_DPI = 200
FIG_W = 9.6  # inches; the SVG is 960 px wide at 100 px per inch

# Reference palette of the dataviz skill; every hex below is a documented step.
THEMES = {
    "light": {
        "surface": "#fcfcfb",
        "ink": "#0b0b0b",
        "ink2": "#52514e",
        "muted": "#898781",
        "grid": "#e1e0d9",
        "axis": "#c3c2b7",
        "blue": "#2a78d6",
        "red": "#e34948",
        "context": "#c3c2b7",  # de-emphasised marks
        "context_strong": "#898781",
        "ramp": ("#86b6ef", "#2a78d6", "#104281"),  # ordinal, small -> large
        "classes": ("#cde2fb", "#6da7ec", "#184f95"),  # sequential steps 100 / 300 / 600
        "class_ink": ("#0b0b0b", "#0b0b0b", "#ffffff"),
        "negative_alpha": 0.30,
    },
    "dark": {
        "surface": "#1a1a19",
        "ink": "#f0efec",
        "ink2": "#c3c2b7",
        "muted": "#898781",
        "grid": "#2c2c2a",
        "axis": "#383835",
        "blue": "#3987e5",
        "red": "#e66767",
        "context": "#52514e",
        "context_strong": "#898781",
        "ramp": ("#184f95", "#3987e5", "#9ec5f4"),  # ordinal flips in dark: larger = lighter
        "classes": ("#0d366b", "#256abf", "#86b6ef"),  # steps 700 / 500 / 250
        "class_ink": ("#f0efec", "#ffffff", "#0b0b0b"),
        "negative_alpha": 0.38,
    },
}

FOOTER = "Estimates, not quotes. Inputs and sources: tools/bizmodel/inputs.json. Model: tools/bizmodel/model.py."

INPUT_LABELS = {
    "price_reseller_monthly_minimum": "Reseller monthly minimum (EUR)",
    "price_reseller_fee_per_card": "Reseller fee per card (EUR)",
    "price_author_plan": "Author plan price (EUR a year)",
    "price_author_jacket": "Price per jacket in a pack (EUR)",
    "price_mailed_card": "Mailed card price (EUR)",
    "reseller_attach_rate": "Attach rate at resellers",
    "reseller_annual_churn": "Reseller churn per year",
    "reseller_orders_small": "eBook orders a year, small reseller",
    "reseller_orders_mid": "eBook orders a year, mid-size reseller",
    "reseller_orders_large": "eBook orders a year, large reseller",
    "reseller_ramp_months": "Months for a reseller to ramp up",
    "minimal_first_partner_month": "Month the first reseller goes live",
    "minimal_months_between_partners": "Months between two resellers",
    "minimal_max_partners": "Resellers a solo founder can carry",
    "minimal_authors_new_month": "New paying authors a month",
    "author_annual_churn": "Author churn per year",
    "author_packs_per_year": "Packs per author per year",
    "author_market_cap": "Ceiling on paying authors",
    "print_jacket_pack100": "Print cost per jacket (EUR, excl. VAT)",
    "print_variable_data_surcharge": "Unique-code surcharge per jacket (EUR)",
    "pack_shipping": "Shipping per pack (EUR)",
    "setup_counsel_before_reseller": "Lawyer before the first reseller (EUR)",
    "setup_build_completion": "Cash to finish the software (EUR)",
    "setup_legal_templates": "Terms and privacy templates (EUR)",
    "setup_trademark_inpi": "Trade mark filing (EUR)",
    "setup_samples_test_orders": "Samples and test orders (EUR)",
    "tax_micro_social_services": "Social charges on services (micro)",
    "tax_micro_social_goods": "Social charges on goods (micro)",
    "fix_rc_pro_year": "Liability insurance (EUR a year)",
    "fix_cfe_year": "CFE local tax (EUR a year)",
    "fix_hosting_month": "Hosting (EUR a month)",
    "fix_bank_month": "Bank account (EUR a month)",
    "fix_email_month": "Transactional e-mail (EUR a month)",
    "fix_einvoicing_month": "E-invoicing platform (EUR a month)",
    "fix_epr_year": "EPR registration (EUR a year)",
    "fix_domains_year": "Domains (EUR a year)",
    "fix_accountant_micro_year": "Accountant, micro (EUR a year)",
    "season_december_share": "December share of yearly volume",
    "pay_card_pct": "Card fee, variable part",
    "pay_card_fixed": "Card fee, fixed part (EUR)",
    "pay_sepa_cap": "SEPA fee cap (EUR)",
    "b2c_organic_cards_month_start": "Free cards a month at launch",
    "b2c_organic_growth_month": "Organic growth per month",
    "var_infra_per_card": "Hosting cost per card (EUR)",
}


# --------------------------------------------------------------------------- #
# Formatting helpers
# --------------------------------------------------------------------------- #


def setup_matplotlib() -> None:
    available = {font.name for font in font_manager.fontManager.ttflist}
    family = "Inter" if "Inter" in available else "DejaVu Sans"
    plt.rcParams.update(
        {
            "font.family": family,
            "font.size": 9.5,
            "svg.fonttype": "path",
            "svg.hashsalt": "ebook-cadeau-bizmodel",
            "axes.unicode_minus": True,
            "figure.dpi": 100,
        }
    )


def compact(value: float) -> str:
    """1,284 -> '1,284'; 12,900 -> '12.9K'; 4,200,000 -> '4.2M'."""
    magnitude = abs(value)
    if magnitude >= 1e6:
        text = f"{value / 1e6:.1f}M"
    elif magnitude >= 1e4:
        text = f"{value / 1e3:.0f}K"
    elif magnitude >= 1e3:
        text = f"{value:,.0f}"
    else:
        text = f"{value:.0f}"
    return text.replace(".0M", "M")


def k_eur(value: float, signed: bool = False) -> str:
    """Amount in thousands of euros with one decimal below 10K."""
    thousands = value / 1000.0
    digits = 1 if abs(thousands) < 10 else 0
    text = f"{abs(thousands):,.{digits}f}"
    sign = "−" if thousands < 0 and float(text.replace(",", "")) != 0 else ("+" if signed and thousands > 0 else "")
    return f"{sign}{text}K"


def money(value: float, digits: int = 2) -> str:
    sign = "−" if value < 0 else ""
    return f"{sign}{abs(value):,.{digits}f}"


def pct(value: float, digits: int = 0) -> str:
    return f"{value * 100:.{digits}f} %"


def trim(value: float, unit: str = "") -> str:
    """Input value for a tornado label: as short as it can be."""
    if unit.startswith("fraction") or unit.startswith("probability"):
        return f"{value * 100:g} %"
    if abs(value) >= 1000:
        return f"{value:,.0f}"
    return f"{value:g}"


def write_csv(name: str, header: list, rows: list) -> None:
    DATA.mkdir(parents=True, exist_ok=True)
    with open(DATA / f"{name}.csv", "w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(header)
        for row in rows:
            writer.writerow([f"{cell:.4f}" if isinstance(cell, float) else cell for cell in row])


def rounded(value, digits: int = 2):
    if isinstance(value, float):
        if math.isinf(value) or math.isnan(value):
            return None
        return round(value, digits)
    if isinstance(value, dict):
        return {str(key): rounded(item, digits) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [rounded(item, digits) for item in value]
    return value


# --------------------------------------------------------------------------- #
# Drawing helpers
# --------------------------------------------------------------------------- #


def new_figure(t: dict, height: float, title: str, subtitle: str):
    """Figure with a left-aligned question as title, a subtitle and a footer."""
    fig = plt.figure(figsize=(FIG_W, height), dpi=100, facecolor=t["surface"])
    left = 0.35 / FIG_W
    y = 1.0 - 0.30 / height
    for line in textwrap.wrap(title, 88):
        fig.text(left, y, line, fontsize=13.5, fontweight="bold", color=t["ink"], va="top", ha="left")
        y -= 0.26 / height
    y -= 0.05 / height
    for line in textwrap.wrap(subtitle, 128):
        fig.text(left, y, line, fontsize=9.5, color=t["ink2"], va="top", ha="left")
        y -= 0.185 / height
    fig.text(left, 0.13 / height, FOOTER, fontsize=7.5, color=t["muted"], va="bottom", ha="left")
    top_inches = (1.0 - y) * height
    return fig, top_inches


def add_panels(fig, rows: int, cols: int, top: float, bottom: float = 0.75, left: float = 0.95, right: float = 0.35,
               wspace: float = 0.85, hspace: float = 0.95) -> list:
    """Grid of axes; all distances in inches. Returns axes row by row."""
    width, height = fig.get_size_inches()
    panel_w = (width - left - right - wspace * (cols - 1)) / cols
    panel_h = (height - top - bottom - hspace * (rows - 1)) / rows
    axes = []
    for r in range(rows):
        for c in range(cols):
            x = left + c * (panel_w + wspace)
            y = height - top - (r + 1) * panel_h - r * hspace
            axes.append(fig.add_axes([x / width, y / height, panel_w / width, panel_h / height]))
    return axes


def style(ax, t: dict, grid: str = "y") -> None:
    ax.set_facecolor(t["surface"])
    for side in ("top", "right", "left", "bottom"):
        ax.spines[side].set_visible(False)
    ax.tick_params(axis="both", length=0, colors=t["muted"], labelsize=8.5, pad=4)
    ax.grid(False)
    if grid in ("y", "both"):
        ax.grid(axis="y", color=t["grid"], linewidth=0.8, linestyle="-")
    if grid in ("x", "both"):
        ax.grid(axis="x", color=t["grid"], linewidth=0.8, linestyle="-")
    ax.set_axisbelow(True)
    ax.xaxis.label.set_color(t["ink2"])
    ax.yaxis.label.set_color(t["ink2"])
    ax.xaxis.label.set_size(9)
    ax.yaxis.label.set_size(9)


def panel_title(ax, t: dict, text: str, sub: str | None = None) -> None:
    ax.text(0.0, 1.085 if sub else 1.03, text, transform=ax.transAxes, fontsize=10, fontweight="bold", color=t["ink"],
            va="bottom", ha="left")
    if sub:
        ax.text(0.0, 1.02, sub, transform=ax.transAxes, fontsize=8.5, color=t["ink2"], va="bottom", ha="left")


def bar(ax, x0: float, x1: float, y0: float, y1: float, color: str, round_at: str = "", alpha: float = 1.0,
        radius_px: float = 4.0, zorder: float = 3.0) -> None:
    """Rectangle in data coordinates with selected corners rounded (4 px at 100 px/inch).

    ``round_at`` lists the sides whose corners are rounded: any of ``l r t b``
    or ``all``. Axis limits and position must be final before calling.
    """
    fig = ax.figure
    (ax0, ay0), (ax1, ay1) = ax.transData.transform([(x0, y0), (x1, y1)])
    left, right = sorted((ax0 / fig.dpi, ax1 / fig.dpi))
    bottom, top = sorted((ay0 / fig.dpi, ay1 / fig.dpi))
    radius = min(radius_px / 100.0, (right - left) / 2.0, (top - bottom) / 2.0)
    if round_at == "all":
        corners = {"tl", "tr", "bl", "br"}
    else:
        corners = set()
        if "l" in round_at:
            corners |= {"tl", "bl"}
        if "r" in round_at:
            corners |= {"tr", "br"}
        if "t" in round_at:
            corners |= {"tl", "tr"}
        if "b" in round_at:
            corners |= {"bl", "br"}
    k = 0.5523 * radius
    verts, codes = [], []

    def move(x, y):
        verts.append((x, y))
        codes.append(MplPath.MOVETO)

    def line(x, y):
        verts.append((x, y))
        codes.append(MplPath.LINETO)

    def curve(c1, c2, end):
        verts.extend([c1, c2, end])
        codes.extend([MplPath.CURVE4] * 3)

    r = radius
    move(left + (r if "bl" in corners else 0), bottom)
    line(right - (r if "br" in corners else 0), bottom)
    if "br" in corners:
        curve((right - r + k, bottom), (right, bottom + r - k), (right, bottom + r))
    line(right, top - (r if "tr" in corners else 0))
    if "tr" in corners:
        curve((right, top - r + k), (right - r + k, top), (right - r, top))
    line(left + (r if "tl" in corners else 0), top)
    if "tl" in corners:
        curve((left + r - k, top), (left, top - r + k), (left, top - r))
    line(left, bottom + (r if "bl" in corners else 0))
    if "bl" in corners:
        curve((left, bottom + r - k), (left + r - k, bottom), (left + r, bottom))
    verts.append(verts[0])
    codes.append(MplPath.CLOSEPOLY)
    patch = PathPatch(MplPath(verts, codes), transform=fig.dpi_scale_trans, facecolor=color, edgecolor="none",
                      alpha=alpha, zorder=zorder, clip_on=False)
    ax.add_patch(patch)


def legend(ax, t: dict, handles: list, labels: list, loc: str = "upper left", anchor=None, ncol: int = 1, title: str | None = None):
    box = ax.legend(handles, labels, loc=loc, bbox_to_anchor=anchor, ncol=ncol, frameon=False, fontsize=8.5,
                    handlelength=1.6, handletextpad=0.6, columnspacing=1.4, borderaxespad=0.0, labelcolor=t["ink2"],
                    title=title)
    if title:
        box.get_title().set_color(t["ink2"])
        box.get_title().set_fontsize(8.5)
        box._legend_box.align = "left"
    return box


def log_formatter():
    return FuncFormatter(lambda value, _: compact(value))


def save(fig, name: str, theme: str) -> None:
    CHARTS.mkdir(parents=True, exist_ok=True)
    suffix = "" if theme == "light" else "-dark"
    fig.savefig(CHARTS / f"{name}{suffix}.svg", format="svg", facecolor=fig.get_facecolor(), metadata={"Date": None})
    fig.savefig(CHARTS / f"{name}{suffix}.png", format="png", dpi=PNG_DPI, facecolor=fig.get_facecolor(),
                metadata={"Software": None})
    plt.close(fig)


# --------------------------------------------------------------------------- #
# Computation
# --------------------------------------------------------------------------- #


def corner_overrides(inputs: dict, score, keep_decisions: bool = True) -> tuple:
    """Adverse and favourable ends of every uncertain input for a scalar ``score`` (higher = better)."""
    base = model.values(inputs)
    adverse, favourable = {}, {}
    for name, row in inputs.items():
        if row["low"] == row["high"] or (keep_decisions and row["role"] == "decision"):
            continue
        at_low = score(dict(base, **{name: row["low"]}))
        at_high = score(dict(base, **{name: row["high"]}))
        if at_low == at_high:
            continue
        adverse[name] = row["low"] if at_low < at_high else row["high"]
        favourable[name] = row["high"] if at_low < at_high else row["low"]
    return adverse, favourable


def compute(inputs: dict) -> dict:
    """Everything the charts and the report need."""
    v = model.values(inputs)
    data: dict = {"base": v}

    # Unit economics, both regimes.
    data["unit"] = {
        regime: {channel: model.unit_economics(channel, v, regime) for channel in ("a", "b", "c", "d", "d_plan")}
        for regime in ("micro_franchise", "company")
    }

    # Scenario x level: base case, corners, Monte Carlo.
    data["sims"], data["summary"], data["mc"], data["corners"] = {}, {}, {}, {}
    for level in model.LEVELS:
        for scenario in model.SCENARIOS:
            key = (level, scenario)
            data["sims"][key] = model.simulate(v, level, scenario)
            data["summary"][key] = model.summarise(data["sims"][key])
            data["mc"][key] = model.monte_carlo(inputs, level, scenario)
            corners = model.corner_cases(inputs, level, scenario)
            data["corners"][key] = {side: model.summarise(result) for side, result in corners.items()}

    data["tornado"] = model.tornado(inputs, "minimal", "MIX")
    data["tornado_seed"] = model.tornado(inputs, "seed", "MIX")

    # Break-even volume against price, minimal level, with cost corners.
    price_ranges = {"a": (0.99, 4.99), "b": (5.0, 12.0), "c": (0.05, 1.0), "d": (1.2, 3.5)}
    data["breakeven"] = {}
    for channel, (low, high) in price_ranges.items():
        adverse, favourable = corner_overrides(inputs, lambda vals, ch=channel: -min(1e12, model.break_even_volume(vals, "minimal", ch)))
        prices = [low + (high - low) * i / 140 for i in range(141)]
        data["breakeven"][channel] = {
            "prices": prices,
            "base": [model.break_even_volume(v, "minimal", channel, p) for p in prices],
            "adverse": [model.break_even_volume(dict(v, **adverse), "minimal", channel, p) for p in prices],
            "favourable": [model.break_even_volume(dict(v, **favourable), "minimal", channel, p) for p in prices],
            "at_base_price": model.break_even_volume(v, "minimal", channel),
            "base_price": v[model.CHANNEL_PRICE_INPUT[channel]],
        }
    data["breakeven_levels"] = {
        level: {channel: model.break_even_volume(v, level, channel) for channel in ("a", "b", "c", "d")}
        for level in model.LEVELS
    }
    data["annual_fixed"] = {
        level: {scenario: model.annual_fixed_cash(v, level, scenario) for scenario in model.SCENARIOS}
        for level in model.LEVELS
    }

    # ROI against steady monthly volume (metered fee only, so the curve is not flattened by the minimum).
    volumes = [10 ** (2 + 3.5 * i / 70) for i in range(71)]
    flat_v = dict(v, price_reseller_monthly_minimum=0.0)
    jackets_per_author_year = v["author_packs_per_year"] * model.PACK_SIZE
    data["roi"] = {"volumes": volumes, "levels": {}}
    for level in model.LEVELS:
        curves = {12: [], 24: [], 36: []}
        for volume in volumes:
            jackets_year = min(0.16 * volume * 12, v["author_market_cap"] * jackets_per_author_year)
            authors = jackets_year / jackets_per_author_year
            reseller_year = volume * 12 - jackets_year
            partners = max(1, round(reseller_year / (v["reseller_orders_mid"] * v["reseller_attach_rate"])))
            flows = model.cash_flow(flat_v, level, "MIX", model.flat_volumes(flat_v, level, reseller_cards_year=reseller_year,
                                                                               partners=partners, authors=authors))
            for horizon in curves:
                curves[horizon].append(model.roi(flows, horizon))
        data["roi"]["levels"][level] = curves

    # Sweet spot: yearly net cash of the minimal set-up by author base and reseller volume.
    author_steps = [0, 25, 50, 100, 200, 400]
    card_steps = [0, 2500, 5000, 10000, 25000, 50000, 100000, 200000]
    data["sweet"] = {
        "authors": author_steps,
        "cards": card_steps,
        "net": [[model.steady_state_net(v, "minimal", cards, authors) for cards in card_steps] for authors in author_steps],
        "growth_threshold": 48 * v["labour_dev_day_rate"],  # one freelance day a week
        "salary_threshold": 12 * v["funded_founder_salary_month"],
        "cards_per_minimum": 12 * v["price_reseller_monthly_minimum"] / v["price_reseller_fee_per_card"],
    }

    # Resellers needed against attach rate.
    rates = [10 ** (math.log10(0.002) + (math.log10(0.10) - math.log10(0.002)) * i / 80) for i in range(81)]
    targets = {
        "salary": 12 * v["funded_founder_salary_month"],
        "team": model.annual_fixed_cash(v, "funded", "C_reseller"),
    }
    data["partners"] = {"rates": rates, "targets": targets, "curves": {}}
    for target_name, target in targets.items():
        data["partners"]["curves"][target_name] = {
            size: [model.partners_needed(v, target, v[f"reseller_orders_{size}"], rate) for rate in rates]
            for size in ("small", "mid", "large")
        }
    return data


# --------------------------------------------------------------------------- #
# Charts
# --------------------------------------------------------------------------- #


def chart_waterfall(data: dict, t: dict):
    v = data["base"]
    panels = [
        ("a", "A'. Paid print-at-home PDF", f"EUR {v['price_paid_pdf']:.2f} per PDF"),
        ("b", "B. Printed-and-posted card", f"EUR {v['price_mailed_card']:.2f} per card, outsourced print and post"),
        ("c", "C. Reseller fee", f"EUR {v['price_reseller_fee_per_card']:.2f} per card generated (minimum ignored)"),
        ("d", "D. Author pack of 100", f"EUR {v['price_author_jacket']:.2f} per jacket"),
    ]
    fig, top = new_figure(
        t, 8.3,
        "What is left of one sale in each channel?",
        "EUR per unit at base inputs, for a solo founder in a micro-entreprise: no VAT charged, VAT on purchases not "
        "recovered, social charges on turnover. Blue: what remains. The last two rows value the founder's own "
        f"support and handling time at EUR {v['founder_hourly_value']:.0f} an hour.",
    )
    axes = add_panels(fig, 2, 2, top=top + 0.55, left=1.75, wspace=2.05, hspace=1.0, bottom=0.8)
    rows_csv = []
    for ax, (channel, title, sub) in zip(axes, panels):
        ue = data["unit"]["micro_franchise"][channel]
        time_cost = ue.minutes / 60.0 * v["founder_hourly_value"]
        steps = list(ue.steps[1:])
        labels = ["Price paid"] + [name for name, _ in steps] + ["Cash contribution", "Founder time", "After founder time"]
        count = len(labels)
        style(ax, t, grid="x")
        low = min(0.0, ue.contribution - time_cost)
        ax.set_xlim(low * 1.05 - 0.02 * ue.price, ue.price * 1.2)
        ax.set_ylim(count - 0.5, -0.5)
        ax.set_yticks(range(count))
        ax.set_yticklabels(labels, fontsize=8.5, color=t["ink2"])
        for label in ax.get_yticklabels():
            if label.get_text() in ("Cash contribution", "After founder time"):
                label.set_color(t["ink"])
                label.set_fontweight("bold")
        ax.set_xlabel("EUR per unit")
        ax.axvline(0, color=t["axis"], linewidth=1.0, zorder=2)
        panel_title(ax, t, title, sub)
        half = 0.27
        pad = ue.price * 0.02

        def value_label(x, row, text, bold=False):
            ax.text(x + pad, row, text, va="center", ha="left", fontsize=8.5, color=t["ink"] if bold else t["ink2"],
                    fontweight="bold" if bold else "normal")

        bar(ax, 0, ue.price, -half, half, t["context_strong"], round_at="r")
        value_label(ue.price, 0, money(ue.price))
        rows_csv.append([channel, "Price paid", ue.price])
        running = ue.price
        row = 1
        for name, amount in steps:
            ax.plot([running, running], [row - 1 + half, row - half], color=t["axis"], linewidth=0.8, zorder=2)
            bar(ax, running + amount, running, row - half, row + half, t["context"], round_at="l")
            value_label(running, row, money(amount))
            rows_csv.append([channel, name, amount])
            running += amount
            row += 1
        colour = t["blue"] if ue.contribution >= 0 else t["red"]
        bar(ax, 0, ue.contribution, row - half, row + half, colour, round_at="r" if ue.contribution >= 0 else "l")
        value_label(max(ue.contribution, 0), row, money(ue.contribution), bold=True)
        rows_csv.append([channel, "Cash contribution", ue.contribution])
        row += 1
        ax.plot([ue.contribution, ue.contribution], [row - 1 + half, row - half], color=t["axis"], linewidth=0.8, zorder=2)
        bar(ax, ue.contribution - time_cost, ue.contribution, row - half, row + half, t["context"], round_at="l")
        value_label(max(ue.contribution, 0), row, money(-time_cost))
        rows_csv.append([channel, "Founder time", -time_cost])
        row += 1
        after = ue.contribution - time_cost
        colour = t["blue"] if after >= 0 else t["red"]
        bar(ax, 0, after, row - half, row + half, colour, round_at="r" if after >= 0 else "l")
        value_label(max(after, 0), row, money(after), bold=True)
        rows_csv.append([channel, "After founder time", after])
    write_csv("01-unit-economics-waterfall", ["channel", "step", "eur_per_unit"], rows_csv)
    return fig


def chart_breakeven(data: dict, t: dict):
    v = data["base"]
    fixed = data["annual_fixed"]["minimal"]
    panels = [
        ("a", "A'. Paid print-at-home PDF", "Price per PDF, EUR incl. VAT", "paid PDFs", ("minimal", "A_fee")),
        ("b", "B. Printed-and-posted card", "Price per card, EUR incl. VAT", "mailed cards", ("minimal", "B_mailed")),
        ("c", "C. Reseller fee", "Fee per card, EUR excl. VAT", "reseller cards", ("minimal", "C_reseller")),
        ("d", "D. Author packs", "Price per jacket, EUR incl. VAT", "jackets", ("minimal", "D_authors")),
    ]
    fig, top = new_figure(
        t, 8.4,
        "How many sales a year must each channel make to pay the bills, and how does the price move that?",
        "Units a year at which one channel alone covers a solo founder's yearly cash costs "
        f"(EUR {min(fixed.values()):,.0f} to {max(fixed.values()):,.0f} depending on the channel; own time unpaid). Line: base inputs. "
        "Band: every cost input at its favourable and at its adverse end. Dot: the base price. Log scale.",
    )
    axes = add_panels(fig, 2, 2, top=top + 0.5, left=0.85, wspace=1.0, hspace=1.15, bottom=0.85)
    rows_csv = []
    for ax, (channel, title, xlabel, unit, sim_key) in zip(axes, panels):
        series = data["breakeven"][channel]
        prices = series["prices"]
        ceiling = 1e6
        finite = [value for value in series["favourable"] if value < ceiling]
        floor = 10 ** math.floor(math.log10(min(finite) * 0.8))
        style(ax, t, grid="y")
        ax.set_yscale("log")
        ax.set_ylim(floor, ceiling)
        ax.set_xlim(prices[0], prices[-1])
        ax.yaxis.set_major_formatter(log_formatter())
        ax.yaxis.set_minor_locator(NullLocator())
        ax.set_xlabel(xlabel)
        ax.set_ylabel(f"{unit.capitalize()} a year (log)")
        clip = lambda column: [min(value, ceiling * 10) for value in column]  # noqa: E731
        ax.fill_between(prices, clip(series["favourable"]), clip(series["adverse"]), color=t["blue"], alpha=0.12, linewidth=0, zorder=2)
        shown = [(p, value) for p, value in zip(prices, series["base"]) if value < ceiling * 10]
        ax.plot([p for p, _ in shown], [value for _, value in shown], color=t["blue"], linewidth=1.8, zorder=3,
                solid_capstyle="round", solid_joinstyle="round")
        for p, low, base, high in zip(prices, series["favourable"], series["base"], series["adverse"]):
            rows_csv.append([channel, p, None if math.isinf(low) else low, None if math.isinf(base) else base,
                             None if math.isinf(high) else high])

        base_price, at_base = series["base_price"], series["at_base_price"]
        if at_base < ceiling:
            ax.plot([base_price], [at_base], marker="o", markersize=7, color=t["blue"], markeredgecolor=t["surface"],
                    markeredgewidth=1.6, zorder=5)
            ax.annotate(f"EUR {base_price:.2f}: {compact(round(at_base, -1 if at_base < 1e4 else -2))} a year",
                        (base_price, at_base), xytext=(8, 8), textcoords="offset points", fontsize=8.5, color=t["ink"],
                        fontweight="bold", ha="left", va="bottom")
        # Price floor: below it a unit earns nothing.
        floor_price = next((p for p, value in zip(prices, series["base"]) if not math.isinf(value)), None)
        if floor_price is not None and floor_price > prices[0]:
            ax.axvline(floor_price, color=t["muted"], linewidth=0.9, linestyle=(0, (4, 3)), zorder=2)
            ax.text(floor_price, ceiling * 0.6, f" below EUR {floor_price:.2f}\n every sale loses money", fontsize=8,
                    color=t["ink2"], ha="right", va="top", linespacing=1.3)
        # What the base case actually sells in year 3.
        expected = data["summary"][sim_key]["last_year_paying_units"]
        if expected >= floor:
            ax.axhline(expected, color=t["muted"], linewidth=0.9, linestyle=(0, (4, 3)), zorder=2)
            ax.text(prices[-1], expected, f"base-case sales in year 3: {compact(round(expected, -1))} ", fontsize=8,
                    color=t["ink2"], ha="right", va="bottom")
        panel_title(ax, t, title)
    write_csv("02-break-even-volume-vs-price", ["channel", "price_eur", "units_favourable", "units_base", "units_adverse"], rows_csv)
    return fig


def chart_cumulative(data: dict, t: dict):
    fig, top = new_figure(
        t, 5.2,
        "How much cash goes in at each investment level, and does it come back within three years?",
        "Cumulative cash, recommended mix (free generator + authors + resellers). Line: median of 2,000 Monte Carlo draws; "
        "band: 10th to 90th percentile. The founder's time is unpaid at the minimal and seed levels. "
        "The vertical scale differs by panel.",
    )
    axes = add_panels(fig, 1, 3, top=top + 0.6, left=0.75, wspace=0.85, bottom=0.95)
    months = list(range(0, model.MONTHS + 1))
    rows_csv = []
    for ax, level in zip(axes, model.LEVELS):
        mc = data["mc"][(level, "MIX")]
        p10, p50, p90 = ([0.0] + mc["cum_cash"][q] for q in (10, 50, 90))
        scale = 1000.0
        style(ax, t, grid="y")
        ax.fill_between(months, [x / scale for x in p10], [x / scale for x in p90], color=t["blue"], alpha=0.14, linewidth=0, zorder=2)
        ax.plot(months, [x / scale for x in p50], color=t["blue"], linewidth=1.8, zorder=4, solid_capstyle="round")
        ax.axhline(0, color=t["axis"], linewidth=1.0, zorder=3)
        ax.set_xlim(0, 39.5)
        ax.xaxis.set_major_locator(FixedLocator([0, 12, 24, 36]))
        ax.set_xlabel("Months from launch (January 2027)")
        if level == "minimal":
            ax.set_ylabel("Cumulative cash, EUR thousand")
        low, high = min(p10) / scale, max(max(p90) / scale, 0.0)
        span = high - low
        ax.set_ylim(low - 0.16 * span, high + 0.22 * span)
        ax.yaxis.set_major_formatter(FuncFormatter(lambda value, _: f"{value:,.0f}".replace("-", "−")))
        title = {"minimal": "Minimal", "seed": "Seed", "funded": "Funded"}[level]
        sub = {"minimal": "solo founder, no paid acquisition", "seed": "part-time developer + marketing",
               "funded": "salaried team of 2.5 + marketing"}[level]
        panel_title(ax, t, title, sub)

        # Deepest point of the median path = the cash to put in.
        trough = min(range(len(p50)), key=lambda i: p50[i])
        need = mc["peak_cash_need"]
        ax.plot([months[trough]], [p50[trough] / scale], marker="o", markersize=6.5, color=t["blue"],
                markeredgecolor=t["surface"], markeredgewidth=1.6, zorder=5)
        note = f"cash needed: EUR {k_eur(need['p50'])}\n(10–90 %: {k_eur(need['p10'])} to {k_eur(need['p90'])})"
        if level == "minimal":
            ax.annotate(note, (months[trough], p50[trough] / scale), xytext=(0, -9), textcoords="offset points",
                        fontsize=8.2, color=t["ink"], ha="center", va="top", linespacing=1.3)
            payback = model.payback_month(p50[1:])
            if payback:
                ax.plot([payback], [p50[payback] / scale], marker="o", markersize=6.5, color=t["blue"],
                        markeredgecolor=t["surface"], markeredgewidth=1.6, zorder=5)
                ax.annotate(f"paid back:\nmonth {payback}", (payback, p50[payback] / scale), xytext=(-4, 9),
                            textcoords="offset points", fontsize=8.2, color=t["ink"], ha="right", va="bottom", linespacing=1.3)
            ax.annotate(f"EUR {k_eur(p50[-1], signed=True)}", (36, p50[-1] / scale), xytext=(-2, 9), textcoords="offset points",
                        fontsize=8.5, color=t["ink"], fontweight="bold", ha="right", va="bottom")
        else:
            ax.text(0.04, 0.05, note + f"\nprobability of payback by month 36: {pct(mc['prob_payback'])}",
                    transform=ax.transAxes, fontsize=8.2, color=t["ink"], ha="left", va="bottom", linespacing=1.3)
        for month, a, b, c in zip(months, p10, p50, p90):
            rows_csv.append([level, month, a, b, c])
    write_csv("03-cumulative-cash-by-investment-level", ["level", "month", "p10_eur", "median_eur", "p90_eur"], rows_csv)
    return fig


def chart_roi(data: dict, t: dict):
    volumes = data["roi"]["volumes"]
    fig, top = new_figure(
        t, 5.3,
        "How much monthly volume does each investment level need before the money comes back?",
        "ROI = net cash generated ÷ cash committed (set-up, fixed costs, people, marketing); −100 % means all of it is gone, "
        "0 % means it just came back. Steady volume from month 1: reseller cards at the metered fee (no minimum) "
        "plus 16 % author jackets up to the ceiling of paying authors.",
    )
    axes = add_panels(fig, 1, 3, top=top + 0.95, left=0.8, wspace=0.55, bottom=0.95)
    rows_csv = []
    handles = []
    for ax, level in zip(axes, model.LEVELS):
        curves = data["roi"]["levels"][level]
        style(ax, t, grid="y")
        ax.set_xscale("log")
        ax.set_xlim(volumes[0], volumes[-1])
        ax.set_ylim(-110, 210)
        ax.xaxis.set_major_locator(FixedLocator([100, 1000, 10000, 100000]))
        ax.xaxis.set_minor_locator(NullLocator())
        ax.xaxis.set_major_formatter(log_formatter())
        ax.yaxis.set_major_locator(FixedLocator([-100, 0, 100, 200]))
        ax.yaxis.set_major_formatter(FuncFormatter(lambda value, _: f"{value:+.0f} %".replace("-", "−").replace("+0", "0")))
        ax.axhline(0, color=t["axis"], linewidth=1.0, zorder=3)
        ax.set_xlabel("Cards a month, all channels (log)")
        if level == "minimal":
            ax.set_ylabel("ROI on cash committed")
        else:
            ax.tick_params(axis="y", labelleft=False)
        handles = []
        for colour, horizon in zip(t["ramp"], (12, 24, 36)):
            values = [min(260.0, 100.0 * value) for value in curves[horizon]]
            (line,) = ax.plot(volumes, values, color=colour, linewidth=1.8, zorder=4, solid_capstyle="round",
                              solid_joinstyle="round")
            handles.append(line)
        # Break-even volume at 36 months.
        curve = curves[36]
        crossing = None
        for i in range(1, len(volumes)):
            if curve[i - 1] < 0 <= curve[i]:
                share = -curve[i - 1] / (curve[i] - curve[i - 1])
                crossing = 10 ** (math.log10(volumes[i - 1]) + share * (math.log10(volumes[i]) - math.log10(volumes[i - 1])))
                break
        if crossing:
            ax.plot([crossing], [0], marker="o", markersize=7, color=t["ramp"][2], markeredgecolor=t["surface"],
                    markeredgewidth=1.6, zorder=6)
            digits = -2 if crossing >= 1000 else -1
            label = f"money back in 36 months\nfrom ≈ {compact(round(crossing, digits))} cards a month"
            right_side = crossing < 3000
            ax.annotate(label, (crossing, 0), xytext=(9 if right_side else -9, -10), textcoords="offset points", fontsize=8.2,
                        color=t["ink"], ha="left" if right_side else "right", va="top", linespacing=1.3)
        title = {"minimal": "Minimal", "seed": "Seed", "funded": "Funded"}[level]
        committed = data["summary"][(level, "MIX")]["outlay_total"]
        panel_title(ax, t, title, f"about EUR {k_eur(committed)} committed over 36 months")
        for i, volume in enumerate(volumes):
            rows_csv.append([level, volume, curves[12][i], curves[24][i], curves[36][i]])
    fig.legend(handles, ["after 12 months", "after 24 months", "after 36 months"], loc="upper left",
               bbox_to_anchor=(0.8 / FIG_W, 1.0 - (top + 0.12) / 5.3), ncol=3, frameon=False, fontsize=8.5, handlelength=1.8,
               columnspacing=1.6, labelcolor=t["ink2"], borderaxespad=0.0)
    write_csv("04-roi-vs-monthly-volume", ["level", "cards_per_month", "roi_12", "roi_24", "roi_36"], rows_csv)
    return fig


def chart_tornado(data: dict, t: dict):
    rows = data["tornado"][:12]
    base = rows[0]["base_value"]
    fig, top = new_figure(
        t, 6.4,
        "Which unknowns and which prices move the three-year result most?",
        f"Cumulative cash after 36 months, minimal level, recommended mix; base case EUR {base:,.0f}. Each bar moves one input "
        "from its low to its high value and leaves the rest at base. The figures at the bar ends are the input values. "
        "Rows marked 'choice' are prices the owner sets.",
    )
    (ax,) = add_panels(fig, 1, 1, top=top + 0.55, left=3.25, right=0.9, bottom=0.9)
    style(ax, t, grid="x")
    low = min(min(row["value_at_low"], row["value_at_high"]) for row in rows)
    high = max(max(row["value_at_low"], row["value_at_high"]) for row in rows)
    span = high - low
    ax.set_xlim(low - 0.13 * span, high + 0.13 * span)
    ax.set_ylim(len(rows) - 0.5, -0.5)
    ax.set_yticks(range(len(rows)))
    names = []
    for row in rows:
        name = INPUT_LABELS.get(row["input"], row["input"].replace("_", " "))
        names.append(name + ("  · choice" if row["role"] == "decision" else ""))
    ax.set_yticklabels(names, fontsize=8.8, color=t["ink2"])
    ax.set_xlabel("Cumulative cash after 36 months, EUR")
    ax.xaxis.set_major_formatter(FuncFormatter(lambda value, _: f"{value:,.0f}".replace("-", "−")))
    ax.axvline(base, color=t["ink2"], linewidth=1.0, zorder=4)
    ax.text(base, -0.62, f"base {base:,.0f}", fontsize=8.2, color=t["ink2"], ha="center", va="bottom")
    half = 0.28
    rows_csv = []
    for index, row in enumerate(rows):
        for value, label in ((row["value_at_low"], trim(row["low"], row["unit"])), (row["value_at_high"], trim(row["high"], row["unit"]))):
            if abs(value - base) < 1e-9:
                continue
            better = value > base
            bar(ax, base, value, index - half, index + half, t["blue"] if better else t["red"], round_at="r" if better else "l")
            ax.annotate(label, (value, index), xytext=(5 if better else -5, 0), textcoords="offset points", fontsize=8.2,
                        color=t["ink"], ha="left" if better else "right", va="center")
        rows_csv.append([row["input"], row["role"], row["low"], row["high"], row["value_at_low"], row["value_at_high"], base])
    handles = [plt.Rectangle((0, 0), 1, 1, color=t["red"]), plt.Rectangle((0, 0), 1, 1, color=t["blue"])]
    legend(ax, t, handles, ["worse than base", "better than base"], loc="lower right", anchor=(1.0, 0.02))
    write_csv("05-sensitivity-tornado",
              ["input", "role", "low", "high", "cum_cash_36_at_low", "cum_cash_36_at_high", "cum_cash_36_base"], rows_csv)
    return fig


def chart_sweet_spot(data: dict, t: dict):
    sweet = data["sweet"]
    v = data["base"]
    need = data["mc"][("minimal", "MIX")]["peak_cash_need"]
    base = data["summary"][("minimal", "MIX")]
    fig, top = new_figure(
        t, 5.9,
        "At what volume and channel mix does a minimal set-up fund itself, and then fund its own growth?",
        f"Yearly net cash of a solo founder (about EUR {k_eur(need['p10'])} to {k_eur(need['p90'])} put in, own time unpaid), by number of "
        f"paying authors and by reseller volume at EUR {v['price_reseller_fee_per_card']:.2f} a card. A reseller on the "
        f"EUR {v['price_reseller_monthly_minimum']:.0f} monthly minimum counts like {sweet['cards_per_minimum']:,.0f} cards a year. "
        f"Base case, year 3: {base['authors_end']:.0f} authors and {base['partners_end']:.1f} resellers, "
        f"about EUR {k_eur(base['last_year_net_cash'], signed=True)}.",
    )
    (ax,) = add_panels(fig, 1, 1, top=top + 0.85, left=1.25, right=0.35, bottom=0.85)
    style(ax, t, grid="")
    authors, cards, net = sweet["authors"], sweet["cards"], sweet["net"]
    ax.set_xlim(-0.5, len(cards) - 0.5)
    ax.set_ylim(-0.5, len(authors) - 0.5)
    ax.set_xticks(range(len(cards)))
    ax.set_xticklabels([f"{c:,.0f}" for c in cards], fontsize=8.8)
    ax.set_yticks(range(len(authors)))
    ax.set_yticklabels([str(a) for a in authors], fontsize=8.8)
    ax.set_xlabel("Cards a year generated through resellers")
    ax.set_ylabel("Paying authors")
    thresholds = (0.0, sweet["growth_threshold"], sweet["salary_threshold"])
    gap = 0.03
    rows_csv = []
    for r, author_count in enumerate(authors):
        for c, card_count in enumerate(cards):
            value = net[r][c]
            if value < thresholds[0]:
                colour, alpha, ink = t["red"], t["negative_alpha"], t["ink"]
            else:
                tier = 0 if value < thresholds[1] else (1 if value < thresholds[2] else 2)
                colour, alpha, ink = t["classes"][tier], 1.0, t["class_ink"][tier]
            bar(ax, c - 0.5 + gap, c + 0.5 - gap, r - 0.5 + gap * 1.6, r + 0.5 - gap * 1.6, colour, round_at="all", alpha=alpha)
            ax.text(c, r, k_eur(value, signed=True), ha="center", va="center", fontsize=9, color=ink, zorder=6)
            rows_csv.append([author_count, card_count, value])
    labels = [
        "does not cover its cash costs",
        "covers its cash costs",
        f"funds growth: a freelance day a week (EUR {sweet['growth_threshold'] / 1000:.0f}K a year)",
        f"pays one founder salary (EUR {sweet['salary_threshold'] / 1000:.0f}K a year)",
    ]
    handles = [plt.Rectangle((0, 0), 1, 1, color=t["red"], alpha=t["negative_alpha"])] + [
        plt.Rectangle((0, 0), 1, 1, color=colour) for colour in t["classes"]
    ]
    fig.legend(handles, labels, loc="upper left", bbox_to_anchor=(1.25 / FIG_W, 1.0 - (top + 0.1) / 5.9), ncol=2, frameon=False,
               fontsize=8.5, handlelength=1.3, handleheight=1.0, columnspacing=2.0, labelcolor=t["ink2"], borderaxespad=0.0)
    write_csv("06-sweet-spot", ["paying_authors", "reseller_cards_per_year", "net_cash_per_year_eur"], rows_csv)
    return fig


def chart_partners(data: dict, t: dict):
    partners = data["partners"]
    v = data["base"]
    rates = [100 * rate for rate in partners["rates"]]
    fig, top = new_figure(
        t, 5.5,
        "How many reseller partners does it take, depending on how often buyers pick the gift card?",
        f"Resellers needed at EUR {v['price_reseller_fee_per_card']:.2f} a card with a EUR {v['price_reseller_monthly_minimum']:.0f} "
        "monthly minimum. Attach rate = share of a reseller's eBook orders that use the card; the planning range is "
        "0.5 % to 5 % (shaded) and has no source. Where a line is flat, every reseller is paying only the minimum. Log scales.",
    )
    axes = add_panels(fig, 1, 2, top=top + 0.95, left=0.8, wspace=0.75, bottom=0.9, right=0.5)
    panels = [
        ("salary", "To pay one founder salary", f"EUR {partners['targets']['salary'] / 1000:.0f}K a year"),
        ("team", "To carry the funded team", f"EUR {partners['targets']['team'] / 1000:.0f}K a year, reseller channel alone"),
    ]
    sizes = [("small", "small"), ("mid", "mid-size"), ("large", "large")]
    rows_csv = []
    handles = []
    for ax, (key, title, sub) in zip(axes, panels):
        style(ax, t, grid="y")
        ax.set_xscale("log")
        ax.set_yscale("log")
        ax.set_xlim(rates[0], rates[-1])
        ax.set_ylim(0.3, 300)
        ax.xaxis.set_major_locator(FixedLocator([0.2, 0.5, 1, 2, 5, 10]))
        ax.xaxis.set_minor_locator(NullLocator())
        ax.xaxis.set_major_formatter(FuncFormatter(lambda value, _: f"{value:g} %"))
        ax.yaxis.set_major_locator(FixedLocator([1, 3, 10, 30, 100, 300]))
        ax.yaxis.set_minor_locator(NullLocator())
        ax.yaxis.set_major_formatter(FuncFormatter(lambda value, _: f"{value:g}"))
        ax.set_xlabel("Attach rate (log)")
        if key == "salary":
            ax.set_ylabel("Resellers needed (log)")
        ax.axvspan(0.5, 5.0, color=t["context"], alpha=0.18, linewidth=0, zorder=1)
        ax.axvline(2.0, color=t["muted"], linewidth=0.9, linestyle=(0, (4, 3)), zorder=2)
        ax.text(2.0, 300 * 0.82, " base 2 %", fontsize=8, color=t["ink2"], ha="left", va="top")
        platforms = v["seed_max_partners"]
        ax.axhline(platforms, color=t["muted"], linewidth=0.9, linestyle=(0, (4, 3)), zorder=2)
        ax.text(rates[0] * 1.06, platforms * 1.1, f"about {platforms:.0f} eBook platforms in France (assumption)", fontsize=8,
                color=t["ink2"], ha="left", va="bottom")
        handles = []
        for colour, (size, _) in zip(t["ramp"], sizes):
            curve = partners["curves"][key][size]
            (line,) = ax.plot(rates, curve, color=colour, linewidth=1.8, zorder=4, solid_capstyle="round")
            handles.append(line)
            at_base = model.partners_needed(v, partners["targets"][key], v[f"reseller_orders_{size}"], v["reseller_attach_rate"])
            ax.plot([2.0], [at_base], marker="o", markersize=6.5, color=colour, markeredgecolor=t["surface"],
                    markeredgewidth=1.6, zorder=5)
            ax.annotate(f"{at_base:.0f}" if at_base >= 10 else f"{at_base:.1f}", (2.0, at_base), xytext=(7, 4),
                        textcoords="offset points", fontsize=8.5, color=t["ink"], fontweight="bold", ha="left", va="bottom")
        panel_title(ax, t, title, sub)
        for i, rate in enumerate(partners["rates"]):
            rows_csv.append([key, rate] + [partners["curves"][key][size][i] for size, _ in sizes])
    names = [
        f"{label} reseller, {v[f'reseller_orders_{size}'] / 1000:,.0f}K eBook orders a year" for size, label in sizes
    ]
    fig.legend(handles, names, loc="upper left", bbox_to_anchor=(0.8 / FIG_W, 1.0 - (top + 0.12) / 5.5), ncol=3, frameon=False,
               fontsize=8.5, handlelength=1.8, columnspacing=1.6, labelcolor=t["ink2"], borderaxespad=0.0)
    write_csv("07-reseller-partners-vs-attach-rate",
              ["target", "attach_rate", "partners_small", "partners_mid", "partners_large"], rows_csv)
    return fig


CHART_FUNCTIONS = [
    ("01-unit-economics-waterfall", chart_waterfall),
    ("02-break-even-volume-vs-price", chart_breakeven),
    ("03-cumulative-cash-by-investment-level", chart_cumulative),
    ("04-roi-vs-monthly-volume", chart_roi),
    ("05-sensitivity-tornado", chart_tornado),
    ("06-sweet-spot", chart_sweet_spot),
    ("07-reseller-partners-vs-attach-rate", chart_partners),
]


# --------------------------------------------------------------------------- #
# results.json
# --------------------------------------------------------------------------- #


def build_results(inputs: dict, data: dict) -> dict:
    v = data["base"]
    unit = {}
    for regime, channels in data["unit"].items():
        unit[regime] = {
            channel: {
                "unit": model.CHANNEL_UNIT[channel],
                "price": ue.price,
                "steps": [[name, amount] for name, amount in ue.steps],
                "contribution": ue.contribution,
                "founder_minutes": ue.minutes,
                "contribution_after_founder_time": ue.contribution - ue.minutes / 60.0 * v["founder_hourly_value"],
            }
            for channel, ue in channels.items()
        }
    scenarios = {}
    for (level, scenario), summary in data["summary"].items():
        mc = data["mc"][(level, scenario)]
        corners = data["corners"][(level, scenario)]
        scenarios.setdefault(level, {})[scenario] = {
            "label": model.SCENARIOS[scenario].label,
            "base": summary,
            "monte_carlo": {key: value for key, value in mc.items() if key != "cum_cash"},
            "corner_adverse": {key: corners["adverse"][key] for key in ("cum_cash_36", "peak_cash_need", "last_year_net_cash")},
            "corner_favourable": {key: corners["favourable"][key] for key in ("cum_cash_36", "peak_cash_need", "last_year_net_cash", "payback_month")},
        }
    results = {
        "generated_by": "tools/bizmodel/make_charts.py",
        "monte_carlo": {"draws": model.DEFAULT_DRAWS, "seed": model.DEFAULT_SEED, "distribution": "independent triangular over low / base / high"},
        "horizon_months": model.MONTHS,
        "inputs_count": len(inputs),
        "unit_economics": unit,
        "annual_fixed_cash": data["annual_fixed"],
        "break_even_units_per_year": data["breakeven_levels"],
        "scenarios": scenarios,
        "tornado_minimal_mix": data["tornado"][:15],
        "tornado_seed_mix": data["tornado_seed"][:10],
        "sweet_spot": data["sweet"],
        "partners_needed_at_base_attach": {
            target: {
                size: model.partners_needed(v, amount, v[f"reseller_orders_{size}"], v["reseller_attach_rate"])
                for size in ("small", "mid", "large")
            }
            for target, amount in data["partners"]["targets"].items()
        },
        "partner_year_income_at_base_attach": {
            size: model.partner_year_income(v, v[f"reseller_orders_{size}"], v["reseller_attach_rate"]) for size in ("small", "mid", "large")
        },
    }
    return rounded(results)


def main() -> None:
    setup_matplotlib()
    inputs = model.load_inputs()
    data = compute(inputs)
    OUT.mkdir(parents=True, exist_ok=True)
    with open(OUT / "results.json", "w", encoding="utf-8") as handle:
        json.dump(build_results(inputs, data), handle, indent=1, ensure_ascii=False, sort_keys=True)
        handle.write("\n")
    for theme in ("light", "dark"):
        for name, function in CHART_FUNCTIONS:
            save(function(data, THEMES[theme]), name, theme)
    print(f"Wrote {len(CHART_FUNCTIONS) * 4} chart files to {CHARTS.relative_to(ROOT)}/, "
          f"{len(CHART_FUNCTIONS)} tables to {DATA.relative_to(ROOT)}/ and {(OUT / 'results.json').relative_to(ROOT)}")


if __name__ == "__main__":
    main()
