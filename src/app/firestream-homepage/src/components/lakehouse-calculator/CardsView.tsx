"use client";

import { StackCard } from "./StackCard";
import {
  cheapest,
  seatsUnlimited,
  type Period,
  type StackTotals,
} from "./compute";
import { RATES, STACK_ORDER } from "./rates";

interface CardsViewProps {
  totals: StackTotals;
  microsoftF64: boolean;
  period: Period;
}

export function CardsView({ totals, microsoftF64, period }: CardsViewProps) {
  const cheapestKey = cheapest(totals);

  return (
    <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
      {STACK_ORDER.map((key) => (
        <StackCard
          key={key}
          stack={RATES[key]}
          breakdown={totals[key]}
          isCheapest={key === cheapestKey}
          seatsAreUnlimited={seatsUnlimited(RATES[key], { microsoftF64 })}
          period={period}
        />
      ))}
    </div>
  );
}
