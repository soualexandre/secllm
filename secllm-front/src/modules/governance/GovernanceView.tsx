"use client";

import { useState, useEffect } from "react";
import { Card, Button, Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { useGovernanceGlobal, usePutGovernanceGlobal } from "@/hooks/useGovernance";

const defaultPolicy = {
  mask_pii: [] as string[],
  mask_response: true,
  rate_limits: {},
  allowed_models: [] as string[],
  blocked_terms: [] as string[],
  data_redaction: {},
};

export function GovernanceView() {
  const [json, setJson] = useState(JSON.stringify(defaultPolicy, null, 2));
  const { data, isLoading, isError } = useGovernanceGlobal();
  const putGlobal = usePutGovernanceGlobal();

  useEffect(() => {
    if (data) setJson(JSON.stringify(data, null, 2));
  }, [data]);

  function handleSave() {
    try {
      const parsed = JSON.parse(json) as typeof defaultPolicy;
      putGlobal.mutate(parsed);
    } catch {
      // invalid JSON
    }
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-semibold">Governance</h2>
      <Tabs defaultValue="global">
        <TabsList>
          <TabsTrigger value="global">Global</TabsTrigger>
          <TabsTrigger value="client">Client</TabsTrigger>
        </TabsList>
        <TabsContent value="global" className="mt-4">
          <Card title="Global policy">
            {isLoading && <p className="text-muted-foreground text-sm">Loading…</p>}
            {isError && <p className="text-danger text-sm">Failed to load policy.</p>}
            {!isLoading && !isError && (
              <>
                <textarea
                  className="w-full h-64 px-4 py-3 rounded-lg bg-background border border-border font-mono text-sm"
                  value={json}
                  onChange={(e) => setJson(e.target.value)}
                  spellCheck={false}
                />
                <div className="mt-4 flex gap-2">
                  <Button onClick={handleSave} disabled={putGlobal.isPending}>
                    {putGlobal.isPending ? "Saving…" : "Save"}
                  </Button>
                  {putGlobal.isError && (
                    <p className="text-danger text-sm">{putGlobal.error?.message}</p>
                  )}
                </div>
              </>
            )}
          </Card>
        </TabsContent>
        <TabsContent value="client" className="mt-4">
          <Card title="Client policy">
            <p className="text-muted-foreground text-sm">Select a client to edit its policy.</p>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
