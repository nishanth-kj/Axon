"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Activity, RefreshCw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  fetchNodePage,
  formatNodeAge,
  registryUrl,
  type RegistryNode,
} from "@/lib/registry";

export const LiveNodes = () => {
  const [nodes, setNodes] = useState<RegistryNode[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    try {
      const page = await fetchNodePage(1, 6, "last_seen", "desc");
      setNodes(page.nodes);
      setTotal(page.pagination.total);
      setError(null);
    } catch (cause) {
      setNodes([]);
      setTotal(0);
      setError(cause instanceof Error ? cause.message : "Could not reach the registry.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    const timer = setInterval(load, 8000);
    return () => clearInterval(timer);
  }, []);

  return (
    <section id="network" className="py-24 px-4">
      <div className="container max-w-6xl mx-auto">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 mb-10">
          <div>
            <h2 className="text-3xl md:text-5xl font-heading font-bold mb-4">Live nodes</h2>
            <p className="text-muted-foreground max-w-2xl">
              Relays currently registered with the Axon registry. This view reads the Actix server at {registryUrl}.
            </p>
          </div>
          <div className="flex items-center gap-3">
            <Badge variant="outline" className="px-4 py-2 rounded-xl">
              <span className="w-2 h-2 rounded-full bg-emerald-500 mr-2" />
              {total} registered
            </Badge>
            <Button asChild variant="outline" className="rounded-xl">
              <Link href="/nodes">Open the full list</Link>
            </Button>
          </div>
        </div>

        {error ? (
          <Card className="p-8 text-center text-muted-foreground">
            <p>The node list is unavailable. {error}</p>
            <p className="mt-2 text-sm">Start the registry with <span className="font-mono">cargo run -p server</span> or <span className="font-mono">docker compose up registry</span>.</p>
            <Button onClick={load} variant="outline" size="sm" className="mt-4 gap-2">
              <RefreshCw className="w-4 h-4" />
              Try again
            </Button>
          </Card>
        ) : loading && nodes.length === 0 ? (
          <Card className="p-8 text-center text-muted-foreground">Loading nodes…</Card>
        ) : nodes.length === 0 ? (
          <Card className="p-8 text-center text-muted-foreground">
            No nodes are registered yet. Start a desktop or CLI node and it will appear here.
          </Card>
        ) : (
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {nodes.map((node) => (
              <Card key={node.node_id} className="p-5">
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <p className="font-medium">{node.ip}</p>
                    <p className="text-xs text-muted-foreground mt-1">Port {node.port}</p>
                  </div>
                  <StatusBadge status={node.status} />
                </div>
                <p className="mt-4 font-mono text-xs text-muted-foreground break-all">{node.node_id}</p>
                <p className="mt-3 text-xs text-muted-foreground flex items-center gap-2">
                  <Activity className="w-3.5 h-3.5" />
                  Last seen {formatNodeAge(node.last_seen)}
                </p>
              </Card>
            ))}
          </div>
        )}
      </div>
    </section>
  );
};

function StatusBadge({ status }: { status?: string }) {
  const label = status || "UNKNOWN";
  const tone =
    label === "ACTIVE"
      ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20"
      : label === "PENDING"
        ? "bg-amber-500/10 text-amber-500 border-amber-500/20"
        : "bg-red-500/10 text-red-500 border-red-500/20";
  return (
    <Badge variant="outline" className={`${tone} border`}>
      {label}
    </Badge>
  );
}
