"use client";

import {
  formatBytes,
  formatUSD,
  PERIOD_FACTOR,
  PERIOD_SUFFIX,
  type Period,
  type StackTotals,
} from "./compute";

interface NarrativeLineProps {
  storedGB: number;
  processedGB: number;
  seats: number;
  totals: StackTotals;
  period: Period;
}

export function NarrativeLine({
  storedGB,
  processedGB,
  seats,
  totals,
  period,
}: NarrativeLineProps) {
  const factor = PERIOD_FACTOR[period];
  const suffix = PERIOD_SUFFIX[period];
  const msTotal = totals.microsoft.total * factor;
  const fsTotal = totals.firestream.total * factor;
  const pct =
    msTotal > 0 ? Math.round(((msTotal - fsTotal) / msTotal) * 100) : 0;

  return (
    <p className="text-sm leading-relaxed text-muted-foreground md:text-base">
      At <span className="tabular-nums text-foreground">{formatBytes(storedGB)}</span>{" "}
      stored,{" "}
      <span className="tabular-nums text-foreground">{formatBytes(processedGB)}</span>{" "}
      processed/mo, and{" "}
      <span className="tabular-nums text-foreground">
        {seats.toLocaleString("en-US")}
      </span>{" "}
      {seats === 1 ? "person" : "people"} — the same lakehouse costs{" "}
      <span className="tabular-nums text-foreground">
        {formatUSD(msTotal)}
        {suffix}
      </span>{" "}
      on Microsoft and{" "}
      <span className="tabular-nums text-foreground">
        {formatUSD(fsTotal)}
        {suffix}
      </span>{" "}
      on Firestream. That's a{" "}
      <strong className="font-semibold text-emerald-600 dark:text-emerald-400">
        {pct}% difference
      </strong>{" "}
      for identical capability.
    </p>
  );
}
