"use client";

import { Card } from "@/components/ui";

export function BillingView() {
  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-semibold">Billing Analytics</h2>
      <div className="grid gap-4 md:grid-cols-2">
        <Card title="Consumption by client">
          <p className="text-muted-foreground text-sm">Charts will appear here when billing data is available.</p>
        </Card>
        <Card title="Cost estimate">
          <p className="text-muted-foreground text-sm">Cost metrics will appear here.</p>
        </Card>
      </div>
      <Card title="Billing logs">
        <p className="text-muted-foreground text-sm">Use POST /api/v1/billing/logs to add logs. List/analytics endpoint can be added to the backend for full dashboard.</p>
      </Card>
    </div>
  );
}
