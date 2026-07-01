"use client";

import { useMemo, useState } from "react";
import { Info } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { Sidebar, MobileInputs } from "./Sidebar";
import { NarrativeLine } from "./NarrativeLine";
import { CardsView } from "./CardsView";
import { ChartView } from "./ChartView";
import { MethodologySection } from "./MethodologySection";
import { DEFAULT_INPUTS, RATES_AS_OF } from "./rates";
import { computeAll, type Period, PERIOD_LABEL } from "./compute";

const PERIODS: Period[] = ["monthly", "annual", "tco5"];

function PeriodToggle({
  value,
  onChange,
}: {
  value: Period;
  onChange: (p: Period) => void;
}) {
  return (
    <div
      role="radiogroup"
      aria-label="Cost period"
      className="inline-flex items-center gap-0.5 rounded-md border border-border/60 bg-card/40 p-0.5"
    >
      {PERIODS.map((p) => {
        const selected = p === value;
        return (
          <button
            key={p}
            type="button"
            role="radio"
            aria-checked={selected}
            onClick={() => onChange(p)}
            className={cn(
              "rounded-[5px] px-2.5 py-1 text-xs font-medium transition-colors",
              selected
                ? "bg-foreground text-background"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            {PERIOD_LABEL[p]}
          </button>
        );
      })}
    </div>
  );
}

export function LakehouseCalculator() {
  const [storedGB, setStoredGB] = useState<number>(DEFAULT_INPUTS.storedGB);
  const [processedGB, setProcessedGB] = useState<number>(
    DEFAULT_INPUTS.processedGB
  );
  const [seats, setSeats] = useState<number>(DEFAULT_INPUTS.seats);
  const [microsoftF64, setMicrosoftF64] = useState<boolean>(false);
  const [view, setView] = useState<"cards" | "chart">("cards");
  const [period, setPeriod] = useState<Period>("tco5");

  const totals = useMemo(
    () => computeAll({ storedGB, processedGB, seats }, { microsoftF64 }),
    [storedGB, processedGB, seats, microsoftF64]
  );

  const reset = () => {
    setStoredGB(DEFAULT_INPUTS.storedGB);
    setProcessedGB(DEFAULT_INPUTS.processedGB);
    setSeats(DEFAULT_INPUTS.seats);
    setMicrosoftF64(false);
  };

  const sidebarProps = {
    storedGB,
    processedGB,
    seats,
    microsoftF64,
    onStoredChange: setStoredGB,
    onProcessedChange: setProcessedGB,
    onSeatsChange: setSeats,
    onMicrosoftF64Change: setMicrosoftF64,
    onReset: reset,
  };

  return (
    <Tabs
      value={view}
      onValueChange={(v) => setView(v as "cards" | "chart")}
      className="relative min-h-screen bg-background text-foreground"
    >
      <header className="sticky top-0 z-40 border-b border-border/60 bg-background/80 backdrop-blur-md">
        <div className="mx-auto flex h-14 max-w-[1400px] items-center justify-between gap-4 px-4 sm:px-6">
          <div className="flex items-center gap-2">
            <span
              aria-hidden
              className="h-2 w-2 rounded-full bg-emerald-500"
            />
            <span className="text-sm font-semibold tracking-tight text-foreground">
              Firestream
            </span>
            <span className="hidden text-xs text-muted-foreground sm:inline">
              / Lakehouse TCO
            </span>
          </div>
          <div className="flex items-center gap-3">
            <PeriodToggle value={period} onChange={setPeriod} />
            <Separator orientation="vertical" className="h-5" />
            <TabsList className="h-9">
              <TabsTrigger value="cards" className="text-xs">
                Cards
              </TabsTrigger>
              <TabsTrigger value="chart" className="text-xs">
                Chart
              </TabsTrigger>
            </TabsList>
          </div>
        </div>
      </header>

      <MobileInputs {...sidebarProps} />

      <div className="mx-auto flex max-w-[1400px] gap-10 px-4 sm:px-6 lg:px-8">
        <Sidebar {...sidebarProps} />

        <main className="min-w-0 flex-1 space-y-10 py-8 md:py-12">
          <section className="space-y-3">
            <div className="inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-card/50 px-2.5 py-0.5 text-[10px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
              <span className="h-1 w-1 rounded-full bg-emerald-500" />
              Data Lakehouse TCO
            </div>
            <h1 className="text-3xl font-semibold tracking-tight text-foreground md:text-4xl">
              Why pay more for the same lakehouse?
            </h1>
            <p className="max-w-2xl text-sm text-muted-foreground md:text-base">
              Storage and compute are commodities. See the math behind four
              vendor stacks at your scale.
            </p>
          </section>

          <NarrativeLine
            storedGB={storedGB}
            processedGB={processedGB}
            seats={seats}
            totals={totals}
            period={period}
          />

          <TabsContent value="cards" className="mt-0">
            <CardsView
              totals={totals}
              microsoftF64={microsoftF64}
              period={period}
            />
          </TabsContent>
          <TabsContent value="chart" className="mt-0">
            <ChartView totals={totals} period={period} />
          </TabsContent>

          <MethodologySection />

          <Alert className="border-border/60 bg-card/40 text-muted-foreground">
            <Info className="h-4 w-4 text-muted-foreground" />
            <AlertDescription>
              Directional estimate for comparison only. Not a quote. Cloud
              pricing changes frequently and varies by region, commitment, and
              negotiated discount. Verify on each vendor's pricing page before
              any decision. Rates as of {RATES_AS_OF}.
            </AlertDescription>
          </Alert>
        </main>
      </div>
    </Tabs>
  );
}
