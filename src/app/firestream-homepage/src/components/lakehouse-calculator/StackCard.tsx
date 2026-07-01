"use client";

import { ExternalLink } from "lucide-react";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import {
  type CostBreakdown,
  formatUSD,
  PERIOD_FACTOR,
  PERIOD_LABEL,
  type Period,
} from "./compute";
import { BRAND, type StackRates } from "./rates";

interface StackCardProps {
  stack: StackRates;
  breakdown: CostBreakdown;
  isCheapest: boolean;
  seatsAreUnlimited: boolean;
  period: Period;
}

const NEUTRAL_TONES = [
  "bg-foreground/85",
  "bg-foreground/60",
  "bg-foreground/40",
  "bg-foreground/25",
];

const EMERALD_TONES = [
  "bg-emerald-500",
  "bg-emerald-500/75",
  "bg-emerald-500/55",
  "bg-emerald-500/35",
];

const PERIOD_EYEBROW: Record<Period, string> = {
  monthly: "Monthly",
  annual: "Annual",
  tco5: "5-yr",
};

function CostLine({
  label,
  subLabel,
  toneClass,
  value,
}: {
  label: string;
  subLabel?: string;
  toneClass: string;
  value: string;
}) {
  return (
    <div className="flex items-start justify-between gap-3">
      <div className="flex min-w-0 items-start gap-2">
        <span
          className={cn(
            "mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full",
            toneClass
          )}
        />
        <div className="min-w-0">
          <div className="text-muted-foreground">{label}</div>
          {subLabel && (
            <div className="text-xs leading-snug text-foreground/70">
              {subLabel}
            </div>
          )}
        </div>
      </div>
      <span className="whitespace-nowrap font-medium tabular-nums text-foreground/90">
        {value}
      </span>
    </div>
  );
}

function BreakdownBar({
  breakdown,
  total,
  isCheapest,
}: {
  breakdown: CostBreakdown;
  total: number;
  isCheapest: boolean;
}) {
  const safe = Math.max(total, 1);
  const palette = isCheapest ? EMERALD_TONES : NEUTRAL_TONES;
  const parts = [
    { key: "storage", value: breakdown.storage },
    { key: "processing", value: breakdown.processing },
    { key: "seats", value: breakdown.seats },
    { key: "platform", value: breakdown.platform },
  ];
  return (
    <div
      className="flex h-1 w-full overflow-hidden rounded-full bg-border/60"
      role="img"
      aria-label="Cost composition"
    >
      {parts.map((p, i) => (
        <div
          key={p.key}
          className={cn("transition-all duration-300", palette[i])}
          style={{ width: `${(p.value / safe) * 100}%` }}
        />
      ))}
    </div>
  );
}

export function StackCard({
  stack,
  breakdown,
  isCheapest,
  seatsAreUnlimited,
  period,
}: StackCardProps) {
  const brand = BRAND[stack.key];
  const tones = isCheapest ? EMERALD_TONES : NEUTRAL_TONES;
  const factor = PERIOD_FACTOR[period];
  const total = breakdown.total * factor;
  const grounding =
    period === "monthly"
      ? `≈ ${formatUSD(breakdown.annual)}/yr`
      : `≈ ${formatUSD(breakdown.total)}/mo`;

  return (
    <Card
      className={cn(
        "group relative flex h-full flex-col overflow-hidden border-border/60 bg-card p-0 transition-colors duration-200",
        "hover:border-border",
        isCheapest &&
          "border-emerald-500/40 bg-emerald-500/[0.03] dark:border-emerald-400/30 dark:bg-emerald-500/[0.04]"
      )}
    >
      <span
        aria-hidden
        className={cn(
          "absolute left-0 top-6 h-8 w-[2px] rounded-r-full",
          brand.accentBgClass
        )}
      />

      <div className="space-y-1 px-5 pb-3 pt-5">
        <div className="flex items-center justify-between gap-2">
          <a
            href={stack.recommendedStackUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="group/link inline-flex items-center gap-2 rounded-sm outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
          >
            <span
              className={cn(
                "h-1.5 w-1.5 rounded-full",
                brand.accentBgClass
              )}
            />
            <CardTitle className="text-sm font-semibold tracking-tight text-foreground">
              {stack.label}
            </CardTitle>
            <ExternalLink
              aria-hidden
              className="h-3 w-3 text-muted-foreground/60 transition-colors group-hover/link:text-foreground"
            />
            <span className="sr-only">Open reference architecture</span>
          </a>
          {isCheapest && (
            <Badge
              variant="success"
              className="h-5 px-1.5 text-[10px] font-medium tracking-wide"
            >
              Lowest cost
            </Badge>
          )}
        </div>
        <CardDescription className="text-xs leading-relaxed text-muted-foreground/80">
          {stack.sublabel}
        </CardDescription>
      </div>

      <div className="px-5 pb-4 pt-1">
        <div className="text-3xl font-semibold leading-none tracking-tight tabular-nums text-foreground">
          {formatUSD(total)}
        </div>
        <div className="mt-2 text-xs tabular-nums text-muted-foreground">
          {PERIOD_LABEL[period]}
          <span aria-hidden className="mx-1.5 text-border">
            ·
          </span>
          <span>{grounding}</span>
        </div>
      </div>

      <div className="px-5">
        <BreakdownBar
          breakdown={breakdown}
          total={breakdown.total}
          isCheapest={isCheapest}
        />
      </div>

      <div className="flex-1 px-5 pb-5 pt-4">
        <div className="mb-3 text-[10px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
          Recommended stack
          <span aria-hidden className="mx-1.5 text-border">
            ·
          </span>
          <span className="text-muted-foreground/80">
            {PERIOD_EYEBROW[period]}
          </span>
        </div>
        <div className="space-y-2.5 text-sm">
          <CostLine
            label="Storage"
            toneClass={tones[0]}
            value={formatUSD(breakdown.storage * factor)}
          />
          <CostLine
            label="Processing"
            toneClass={tones[1]}
            value={formatUSD(breakdown.processing * factor)}
          />
          <CostLine
            label="Seats"
            toneClass={tones[2]}
            value={
              seatsAreUnlimited
                ? "Unlimited"
                : formatUSD(breakdown.seats * factor)
            }
          />
          <CostLine
            label="Platform"
            subLabel={breakdown.platformLabel}
            toneClass={tones[3]}
            value={formatUSD(breakdown.platform * factor)}
          />
        </div>
      </div>
    </Card>
  );
}
