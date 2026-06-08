"use client";

import { ExternalLink } from "lucide-react";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { BRAND, RATES, RATES_AS_OF, STACK_ORDER } from "./rates";
import { formatBytes, formatUSD } from "./compute";
import { cn } from "@/lib/utils";

function ExtLink({
  href,
  children,
}: {
  href: string;
  children: React.ReactNode;
}) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className="inline-flex items-center gap-1 underline underline-offset-2 hover:text-foreground"
    >
      {children}
      <ExternalLink className="h-3 w-3" aria-hidden />
    </a>
  );
}

export function MethodologySection() {
  return (
    <section
      aria-label="How we calculated this"
      className="border-t border-border/60 pt-10"
    >
      <h2 className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
        Methodology
      </h2>
      <p className="mt-3 max-w-2xl text-sm text-muted-foreground">
        We multiply your sliders by each vendor's published rates and add their
        platform and per-seat fees. Rates as of {RATES_AS_OF}.
      </p>

      <Accordion type="single" collapsible className="mt-4 w-full">
        <AccordionItem value="rates">
          <AccordionTrigger>See the per-vendor rates</AccordionTrigger>
          <AccordionContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Stack</TableHead>
                  <TableHead>Storage $/GB-mo</TableHead>
                  <TableHead>Processing $/GB</TableHead>
                  <TableHead>Seat $/mo</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {STACK_ORDER.map((key) => {
                  const r = RATES[key];
                  const brand = BRAND[key];
                  return (
                    <TableRow key={key}>
                      <TableCell className="font-medium text-foreground">
                        <span className="inline-flex items-center gap-2">
                          <span
                            aria-hidden
                            className={cn(
                              "h-1.5 w-1.5 rounded-full",
                              brand.accentBgClass
                            )}
                          />
                          {r.label}
                        </span>
                      </TableCell>
                      <TableCell className="tabular-nums">
                        ${r.storageRate.toFixed(4)}
                      </TableCell>
                      <TableCell className="tabular-nums">
                        ${r.processingRate.toFixed(4)}
                      </TableCell>
                      <TableCell className="tabular-nums">
                        {r.seatRate === 0
                          ? "$0"
                          : formatUSD(r.seatRate, { decimals: 2 })}
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </AccordionContent>
        </AccordionItem>

        <AccordionItem value="tiers">
          <AccordionTrigger>See how vendor tiers step up</AccordionTrigger>
          <AccordionContent>
            <p className="mb-4 text-sm text-muted-foreground">
              The "data processed/mo" slider picks each vendor's tier
              automatically. Google's Looker platform fee (~$4,000/mo) is
              bundled into the tier price below.
            </p>
            <div className="grid grid-cols-1 gap-5 md:grid-cols-2">
              {STACK_ORDER.map((key) => {
                const r = RATES[key];
                const brand = BRAND[key];
                return (
                  <div
                    key={key}
                    className="rounded-lg border border-border/60 bg-card/40 p-4"
                  >
                    <div className="mb-3 flex items-center justify-between">
                      <h3 className="flex items-center gap-2 text-sm font-semibold text-foreground">
                        <span
                          aria-hidden
                          className={cn(
                            "h-1.5 w-1.5 rounded-full",
                            brand.accentBgClass
                          )}
                        />
                        {r.label}
                      </h3>
                      <span className="text-xs text-muted-foreground">
                        {r.sublabel}
                      </span>
                    </div>
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>Tier</TableHead>
                          <TableHead>Handles up to</TableHead>
                          <TableHead className="text-right">$/mo</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {r.platformTiers.map((t, i) => (
                          <TableRow key={i}>
                            <TableCell className="font-medium text-foreground">
                              {t.label}
                            </TableCell>
                            <TableCell className="tabular-nums text-muted-foreground">
                              {Number.isFinite(t.maxProcessedGB)
                                ? `${formatBytes(t.maxProcessedGB)}/mo`
                                : "beyond"}
                            </TableCell>
                            <TableCell className="text-right tabular-nums">
                              {formatUSD(t.monthly)}
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                );
              })}
            </div>
          </AccordionContent>
        </AccordionItem>

        <AccordionItem value="caveats" className="border-b-0">
          <AccordionTrigger>Assumptions &amp; caveats</AccordionTrigger>
          <AccordionContent>
            <ul className="list-disc space-y-2 pl-5 text-sm text-muted-foreground marker:text-muted-foreground/60">
              <li>
                Amazon QuickSight Enterprise blends Readers ($5/mo cap) with
                Authors ($24/mo) — we use $8/seat as a directional blend
                (~10% authors / 90% readers). Author-heavy teams will pay more;
                read-only deployments at the $5 reader cap will pay less.
              </li>
              <li>
                Microsoft Fabric capacity (F2 → F128) doubles cost and
                throughput per step. Below F64 every viewer also needs a $14/mo
                Power BI Pro seat; at F64+ viewers are free. Toggle{" "}
                <em>Advanced → Force Microsoft F64</em> in the sidebar to see
                that tradeoff at any scale.{" "}
                <ExtLink href="https://learn.microsoft.com/en-us/azure/architecture/example-scenario/data/greenfield-lakehouse-fabric">
                  Fabric reference architecture
                </ExtLink>
                .
              </li>
              <li>
                <strong className="text-foreground">
                  Firestream tier thresholds are engineering estimates
                </strong>{" "}
                of when a single VM runs out of headroom. Real mileage depends
                on workload complexity. The point: self-hosted orchestration
                grows in $150–$1,500 steps on commodity hardware, not in
                $4,000 platform-fee leaps. Excludes self-host ops labor.
              </li>
              <li>
                Google Cloud's reference architecture (linked from the card)
                covers BigQuery, Dataplex, and BigLake for storage and
                governance. We add a Cloud Composer + Looker layer to the cost
                model because real enterprise GCP lakehouses include those for
                orchestration and BI — they're not in the architecture page
                itself but represent the typical full deployment.
              </li>
              <li>
                Firestream's storage and processing rates mirror GCP BigQuery
                and Cloud Storage because the OSS stack still needs
                cloud-managed object storage and a query engine somewhere
                underneath. The savings come from dropping the licensed
                platform fee (Fabric / Looker / QuickSight tiers) and per-seat
                charges — not from compute being cheaper.
              </li>
              <li>
                We don't model reserved or committed-capacity discounts
                (BigQuery slots, Redshift RI, Fabric reserved), region
                multipliers, or negotiated volume. Talk to us before any
                procurement-grade decision.
              </li>
            </ul>
          </AccordionContent>
        </AccordionItem>
      </Accordion>
    </section>
  );
}
