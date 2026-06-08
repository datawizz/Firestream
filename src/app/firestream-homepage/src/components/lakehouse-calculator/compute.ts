import {
  F64_FORCED_TIER,
  RATES,
  STACK_ORDER,
  type PlatformTier,
  type StackKey,
  type StackRates,
} from "./rates";

export interface CalculatorInputs {
  storedGB: number;
  processedGB: number;
  seats: number;
}

export interface CalculatorToggles {
  microsoftF64: boolean;
}

export interface CostBreakdown {
  storage: number;
  processing: number;
  seats: number;
  platform: number;
  platformLabel: string;
  total: number;
  annual: number;
}

export type StackTotals = Record<StackKey, CostBreakdown>;

export type Period = "monthly" | "annual" | "tco5";

export const PERIOD_FACTOR: Record<Period, number> = {
  monthly: 1,
  annual: 12,
  tco5: 60,
};

export const PERIOD_SUFFIX: Record<Period, string> = {
  monthly: "/mo",
  annual: "/yr",
  tco5: " over 5 yrs",
};

export const PERIOD_LABEL: Record<Period, string> = {
  monthly: "Monthly",
  annual: "Annual",
  tco5: "5-yr TCO",
};

// Maps slider position p in [0,1] to a value on a logarithmic scale [min,max].
export function logScale(p: number, min: number, max: number): number {
  const clamped = Math.max(0, Math.min(1, p));
  return min * Math.pow(max / min, clamped);
}

// Inverse of logScale — value -> position. Useful for setting defaults.
export function invLogScale(value: number, min: number, max: number): number {
  const v = Math.max(min, Math.min(max, value));
  return Math.log(v / min) / Math.log(max / min);
}

// Auto-format GB as GB / TB / PB. Execs think in TB+; we never show raw GB.
export function formatBytes(gb: number): string {
  if (gb >= 1_000_000) {
    const pb = gb / 1_000_000;
    return `${formatNumber(pb, pb >= 10 ? 0 : 1)} PB`;
  }
  if (gb >= 1_000) {
    const tb = gb / 1_000;
    return `${formatNumber(tb, tb >= 10 ? 0 : 1)} TB`;
  }
  return `${formatNumber(gb, 0)} GB`;
}

function formatNumber(n: number, decimals: number): string {
  return n.toLocaleString("en-US", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

const usdNoCents = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  minimumFractionDigits: 0,
  maximumFractionDigits: 0,
});
const usdCents = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

export function formatUSD(
  n: number,
  opts: { decimals?: 0 | 2 } = {}
): string {
  const decimals = opts.decimals ?? 0;
  return (decimals === 2 ? usdCents : usdNoCents).format(n);
}

// First tier whose ceiling can absorb processedGB; falls through to the last
// tier (which always has maxProcessedGB = Infinity).
export function pickTier(
  tiers: PlatformTier[],
  processedGB: number
): PlatformTier {
  for (const tier of tiers) {
    if (processedGB <= tier.maxProcessedGB) return tier;
  }
  return tiers[tiers.length - 1];
}

export function computeStack(
  stack: StackRates,
  inputs: CalculatorInputs,
  toggles: CalculatorToggles
): CostBreakdown {
  const isMicrosoftForced = stack.key === "microsoft" && toggles.microsoftF64;
  const tier = isMicrosoftForced
    ? F64_FORCED_TIER
    : pickTier(stack.platformTiers, inputs.processedGB);
  const seatRate = isMicrosoftForced ? 0 : stack.seatRate;

  const storage = inputs.storedGB * stack.storageRate;
  const processing = inputs.processedGB * stack.processingRate;
  const seats = inputs.seats * seatRate;
  const platform = tier.monthly;
  const total = storage + processing + seats + platform;
  return {
    storage,
    processing,
    seats,
    platform,
    platformLabel: tier.label,
    total,
    annual: total * 12,
  };
}

export function computeAll(
  inputs: CalculatorInputs,
  toggles: CalculatorToggles
): StackTotals {
  return STACK_ORDER.reduce((acc, key) => {
    acc[key] = computeStack(RATES[key], inputs, toggles);
    return acc;
  }, {} as StackTotals);
}

export function cheapest(totals: StackTotals): StackKey {
  return STACK_ORDER.reduce((best, key) =>
    totals[key].total < totals[best].total ? key : best
  );
}

// Used by the StackCard "seats are unlimited" indicator: true when the stack's
// effective seatRate is 0 (Looker / Superset / Microsoft-with-F64-forced).
export function seatsUnlimited(
  stack: StackRates,
  toggles: CalculatorToggles
): boolean {
  if (stack.key === "microsoft") {
    return toggles.microsoftF64; // forced F64 → free viewers
  }
  return stack.seatRate === 0;
}
