"use client";

import { Info, RotateCcw } from "lucide-react";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { formatBytes, invLogScale, logScale } from "./compute";
import { DEFAULT_INPUTS, SLIDER_RANGES } from "./rates";

interface SidebarProps {
  storedGB: number;
  processedGB: number;
  seats: number;
  microsoftF64: boolean;
  onStoredChange: (gb: number) => void;
  onProcessedChange: (gb: number) => void;
  onSeatsChange: (seats: number) => void;
  onMicrosoftF64Change: (on: boolean) => void;
  onReset: () => void;
}

function SectionLabel({ children }: { children: React.ReactNode }) {
  return (
    <h2 className="text-[10px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
      {children}
    </h2>
  );
}

function SliderRow({
  label,
  hint,
  displayValue,
  position,
  onPositionChange,
}: {
  label: string;
  hint: string;
  displayValue: string;
  position: number;
  onPositionChange: (p: number) => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5">
          <span className="text-xs font-medium text-foreground">{label}</span>
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                aria-label={`About ${label}`}
                className="text-muted-foreground transition-colors hover:text-foreground"
              >
                <Info className="h-3 w-3" />
              </button>
            </TooltipTrigger>
            <TooltipContent className="max-w-xs">{hint}</TooltipContent>
          </Tooltip>
        </div>
        <span className="text-xs font-semibold tabular-nums text-foreground">
          {displayValue}
        </span>
      </div>
      <Slider
        min={0}
        max={1}
        step={0.001}
        value={[position]}
        onValueChange={(v) => onPositionChange(v[0])}
      />
    </div>
  );
}

function InputsContent({
  storedGB,
  processedGB,
  seats,
  microsoftF64,
  onStoredChange,
  onProcessedChange,
  onSeatsChange,
  onMicrosoftF64Change,
  onReset,
}: SidebarProps) {
  const storedPos = invLogScale(
    storedGB,
    SLIDER_RANGES.stored.min,
    SLIDER_RANGES.stored.max
  );
  const processedPos = invLogScale(
    processedGB,
    SLIDER_RANGES.processed.min,
    SLIDER_RANGES.processed.max
  );
  const seatPos =
    (seats - SLIDER_RANGES.seats.min) /
    (SLIDER_RANGES.seats.max - SLIDER_RANGES.seats.min);

  const isDefault =
    storedGB === DEFAULT_INPUTS.storedGB &&
    processedGB === DEFAULT_INPUTS.processedGB &&
    seats === DEFAULT_INPUTS.seats &&
    !microsoftF64;

  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <SectionLabel>Inputs</SectionLabel>
        <SliderRow
          label="Data stored"
          hint="Stored = the data you keep on disk every month."
          displayValue={formatBytes(storedGB)}
          position={storedPos}
          onPositionChange={(p) =>
            onStoredChange(
              Math.round(
                logScale(p, SLIDER_RANGES.stored.min, SLIDER_RANGES.stored.max)
              )
            )
          }
        />
        <SliderRow
          label="Data processed / month"
          hint="Processed = the data your reports and pipelines read each month."
          displayValue={formatBytes(processedGB)}
          position={processedPos}
          onPositionChange={(p) =>
            onProcessedChange(
              Math.round(
                logScale(
                  p,
                  SLIDER_RANGES.processed.min,
                  SLIDER_RANGES.processed.max
                )
              )
            )
          }
        />
        <SliderRow
          label="People with access"
          hint="Seats = people who open dashboards. Some vendors charge per seat; some don't."
          displayValue={`${seats.toLocaleString("en-US")} seat${seats === 1 ? "" : "s"}`}
          position={seatPos}
          onPositionChange={(p) =>
            onSeatsChange(
              Math.round(
                SLIDER_RANGES.seats.min +
                  p * (SLIDER_RANGES.seats.max - SLIDER_RANGES.seats.min)
              )
            )
          }
        />
      </div>

      <Separator />

      <div className="space-y-3">
        <SectionLabel>Advanced</SectionLabel>
        <label
          htmlFor="f64-toggle"
          className="flex cursor-pointer items-start gap-3 text-xs"
        >
          <Switch
            id="f64-toggle"
            checked={microsoftF64}
            onCheckedChange={onMicrosoftF64Change}
            className="mt-0.5"
          />
          <span className="leading-relaxed">
            <span className="block font-medium text-foreground">
              Force Microsoft F64
            </span>
            <span className="block text-muted-foreground">
              Free Power BI viewers, $5,257/mo floor.
            </span>
          </span>
        </label>
      </div>

      <Separator />

      <Button
        variant="ghost"
        size="sm"
        onClick={onReset}
        disabled={isDefault}
        className="-ml-2 w-full justify-start"
      >
        <RotateCcw className="mr-2 h-3.5 w-3.5" />
        Reset to defaults
      </Button>
    </div>
  );
}

export function Sidebar(props: SidebarProps) {
  return (
    <TooltipProvider delayDuration={200}>
      <aside className="hidden lg:block w-[300px] shrink-0">
        <div className="sticky top-14 py-8">
          <InputsContent {...props} />
        </div>
      </aside>
    </TooltipProvider>
  );
}

export function MobileInputs(props: SidebarProps) {
  return (
    <TooltipProvider delayDuration={200}>
      <div className="sticky top-14 z-30 border-b border-border/60 bg-background/90 backdrop-blur lg:hidden">
        <div className="px-4 sm:px-6">
          <Accordion type="single" collapsible defaultValue="inputs">
            <AccordionItem value="inputs" className="border-b-0">
              <AccordionTrigger className="py-3 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground hover:no-underline">
                Inputs
              </AccordionTrigger>
              <AccordionContent className="pb-5 pt-1">
                <InputsContent {...props} />
              </AccordionContent>
            </AccordionItem>
          </Accordion>
        </div>
      </div>
    </TooltipProvider>
  );
}
