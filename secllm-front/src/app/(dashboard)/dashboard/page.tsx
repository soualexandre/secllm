import { Card } from "@/components/ui";

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-semibold">Dashboard</h2>
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <p className="text-sm text-muted-foreground">Requests/min</p>
          <p className="text-2xl font-semibold">—</p>
        </Card>
        <Card>
          <p className="text-sm text-muted-foreground">Token usage</p>
          <p className="text-2xl font-semibold">—</p>
        </Card>
        <Card>
          <p className="text-sm text-muted-foreground">Latency</p>
          <p className="text-2xl font-semibold">—</p>
        </Card>
        <Card>
          <p className="text-sm text-muted-foreground">Error rate</p>
          <p className="text-2xl font-semibold">—</p>
        </Card>
      </div>
      <Card title="Gateway monitoring">
        <p className="text-muted-foreground text-sm">Metrics and logs will appear here when the backend exposes analytics endpoints.</p>
      </Card>
    </div>
  );
}
