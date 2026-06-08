"use client";

import {
  Bar,
  BarChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  formatUSD,
  PERIOD_FACTOR,
  PERIOD_LABEL,
  PERIOD_SUFFIX,
  type Period,
  type StackTotals,
} from "./compute";
import { BRAND, RATES, STACK_ORDER, type StackKey } from "./rates";

interface ChartViewProps {
  totals: StackTotals;
  period: Period;
}

interface ChartTooltipPayloadItem {
  name?: string | number;
  value?: number;
  color?: string;
  dataKey?: string;
  payload?: ChartDatum;
}

interface ChartTooltipProps {
  active?: boolean;
  payload?: ChartTooltipPayloadItem[];
  label?: string;
}

interface ChartDatum {
  key: StackKey;
  stack: string;
  Storage: number;
  Processing: number;
  Seats: number;
  Platform: number;
  total: number;
}

// Monotonal foreground tones — matches the in-card BreakdownBar so the chart
// and the cards share one visual language.
const COMPONENT_COLORS: Record<string, string> = {
  Storage: "hsl(var(--foreground) / 0.85)",
  Processing: "hsl(var(--foreground) / 0.6)",
  Seats: "hsl(var(--foreground) / 0.4)",
  Platform: "hsl(var(--foreground) / 0.25)",
};

const COMPONENT_ORDER = ["Storage", "Processing", "Seats", "Platform"] as const;

function CustomTooltip({
  active,
  payload,
  label,
  period,
}: ChartTooltipProps & { period: Period }) {
  if (!active || !payload?.length) return null;
  const total = payload[0]?.payload?.total ?? 0;
  const totalLabel =
    period === "monthly"
      ? "Monthly total"
      : period === "annual"
        ? "Annual total"
        : "5-year total";
  return (
    <div className="min-w-[200px] rounded-lg border border-border/60 bg-popover/95 p-3 text-popover-foreground shadow-lg backdrop-blur">
      <div className="mb-2 text-xs font-semibold tracking-tight">{label}</div>
      <div className="space-y-1 text-xs">
        {payload.map((p) => (
          <div
            key={String(p.dataKey)}
            className="flex items-center justify-between gap-4"
          >
            <span className="flex items-center gap-2 text-muted-foreground">
              <span
                aria-hidden
                className="inline-block h-1.5 w-1.5 rounded-full"
                style={{ backgroundColor: p.color }}
              />
              {p.name}
            </span>
            <span className="tabular-nums text-foreground/90">
              {formatUSD(Number(p.value ?? 0))}
            </span>
          </div>
        ))}
      </div>
      <div className="mt-2 flex items-center justify-between border-t border-border/60 pt-2 text-xs font-semibold">
        <span>{totalLabel}</span>
        <span className="tabular-nums">{formatUSD(total)}</span>
      </div>
    </div>
  );
}

function VendorTick(props: {
  x?: number;
  y?: number;
  payload?: { value?: string };
}) {
  const { x, y, payload } = props;
  const label = payload?.value ?? "";
  const stackKey = STACK_ORDER.find((k) => RATES[k].label === label);
  const color = stackKey ? BRAND[stackKey].chartColor : "currentColor";
  return (
    <g transform={`translate(${x ?? 0}, ${(y ?? 0) + 14})`}>
      <circle cx={-6} cy={-3} r={3} fill={color} />
      <text
        x={2}
        y={0}
        textAnchor="middle"
        className="fill-foreground/80 text-[12px]"
      >
        {label}
      </text>
    </g>
  );
}

function ChartLegend() {
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1.5 text-xs text-muted-foreground">
      {COMPONENT_ORDER.map((name) => (
        <span key={name} className="flex items-center gap-2">
          <span
            aria-hidden
            className="inline-block h-1.5 w-1.5 rounded-full"
            style={{ backgroundColor: COMPONENT_COLORS[name] }}
          />
          {name}
        </span>
      ))}
    </div>
  );
}

export function ChartView({ totals, period }: ChartViewProps) {
  const factor = PERIOD_FACTOR[period];
  const data: ChartDatum[] = STACK_ORDER.map((key) => ({
    key,
    stack: RATES[key].label,
    Storage: Math.round(totals[key].storage * factor),
    Processing: Math.round(totals[key].processing * factor),
    Seats: Math.round(totals[key].seats * factor),
    Platform: Math.round(totals[key].platform * factor),
    total: Math.round(totals[key].total * factor),
  }));

  return (
    <div className="rounded-xl border border-border/60 bg-card/40 p-5 md:p-6">
      <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-sm font-semibold tracking-tight text-foreground">
            {PERIOD_LABEL[period]} cost by stack
          </h2>
          <p className="text-xs text-muted-foreground">
            Stacked by cost component
            {period !== "monthly" ? `, totals shown${PERIOD_SUFFIX[period]}.` : "."}
          </p>
        </div>
        <ChartLegend />
      </div>
      <div className="h-[440px] w-full">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart
            data={data}
            margin={{ top: 8, right: 8, bottom: 24, left: 8 }}
          >
            <CartesianGrid
              strokeDasharray="3 3"
              stroke="hsl(var(--border))"
              strokeOpacity={0.5}
              vertical={false}
            />
            <XAxis
              dataKey="stack"
              axisLine={false}
              tickLine={false}
              tick={<VendorTick />}
              height={36}
            />
            <YAxis
              tickFormatter={(v: number) => formatUSD(v)}
              axisLine={false}
              tickLine={false}
              tick={{ fontSize: 11 }}
              stroke="hsl(var(--muted-foreground))"
              width={68}
            />
            <Tooltip
              content={<CustomTooltip period={period} />}
              cursor={{ fill: "hsl(var(--foreground) / 0.04)" }}
            />
            <Bar
              dataKey="Storage"
              stackId="a"
              fill={COMPONENT_COLORS.Storage}
              radius={[0, 0, 0, 0]}
            />
            <Bar
              dataKey="Processing"
              stackId="a"
              fill={COMPONENT_COLORS.Processing}
            />
            <Bar dataKey="Seats" stackId="a" fill={COMPONENT_COLORS.Seats} />
            <Bar
              dataKey="Platform"
              stackId="a"
              fill={COMPONENT_COLORS.Platform}
              radius={[6, 6, 0, 0]}
            />
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
