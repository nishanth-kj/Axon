export const registryUrl =
  process.env.NEXT_PUBLIC_REGISTRY_URL || "http://127.0.0.1:7400";

export type RegistryNode = {
  node_id: string;
  ip: string;
  port: number;
  verification_port?: number;
  status?: string;
  last_seen?: string;
  registered_at?: string;
  failure_count?: number;
};

export type NodePage = {
  nodes: RegistryNode[];
  pagination: {
    total: number;
    page: number;
    size: number;
    totalPages: number;
  };
};

export async function fetchNodePage(
  page: number,
  size: number,
  sortBy: string,
  sortOrder: "asc" | "desc"
): Promise<NodePage> {
  const response = await fetch(`${registryUrl}/api/v1/nodes/details`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ page, size, sortBy, sortOrder }),
  });
  if (!response.ok) {
    throw new Error(`Registry returned HTTP ${response.status}`);
  }
  const result = await response.json();
  if (result.status !== 1 || !result.data) {
    const message = result.error?.error_message || "The registry rejected the node list request.";
    throw new Error(message);
  }
  return {
    nodes: result.data.nodes ?? [],
    pagination: {
      total: result.data.pagination?.total ?? 0,
      page: result.data.pagination?.page ?? page,
      size: result.data.pagination?.size ?? size,
      totalPages: result.data.pagination?.totalPages ?? 0,
    },
  };
}

export function formatNodeTime(value: string | undefined): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

export function formatNodeAge(value: string | undefined): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  const seconds = Math.max(0, Math.round((Date.now() - date.getTime()) / 1000));
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 48) return `${hours}h ago`;
  return date.toLocaleDateString();
}
