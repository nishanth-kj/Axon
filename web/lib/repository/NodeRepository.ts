/**
 * Represents a proxy node registered within the Axon network.
 * Contains connection details, authentication secrets, and health metrics.
 */
export interface ProxyNode {
  node_id: string;
  ip: string;
  port: number;
  status: "ACTIVE" | "OFFLINE" | "PENDING";
  secret_key?: string;
  last_seen: string;
  registered_at: string;
  failure_count: number;
}

/**
 * Defines the standard operations for managing proxy nodes.
 * By abstracting this, the underlying storage (e.g. Memory, Redis, PostgreSQL) 
 * can be easily swapped without affecting the business logic in NodeService.
 */
export interface INodeRepository {
  get(node_id: string): ProxyNode | undefined;
  getAll(): ProxyNode[];
  save(node: ProxyNode): void;
  delete(node_id: string): void;
  findBySecretKey(secretKey: string): ProxyNode | undefined;
}

// In Next.js dev environment, globalThis persists across hot-reloads
declare global {
  var proxyNodesRepository: Map<string, ProxyNode> | undefined;
}

/**
 * An in-memory implementation of the INodeRepository.
 * Uses a global Map to ensure the registry persists across Next.js HMR (Hot Module Replacement)
 * reloads during local development.
 * Note: Not suitable for multi-instance or serverless deployments without a sticky session or single worker.
 */
export class InMemoryNodeRepository implements INodeRepository {
  private store: Map<string, ProxyNode>;

  constructor() {
    if (!globalThis.proxyNodesRepository) {
      globalThis.proxyNodesRepository = new Map();
    }
    this.store = globalThis.proxyNodesRepository;
  }

  get(node_id: string): ProxyNode | undefined {
    return this.store.get(node_id);
  }

  getAll(): ProxyNode[] {
    return Array.from(this.store.values());
  }

  save(node: ProxyNode): void {
    this.store.set(node.node_id, node);
  }

  delete(node_id: string): void {
    this.store.delete(node_id);
  }

  findBySecretKey(secretKey: string): ProxyNode | undefined {
    for (const node of this.store.values()) {
      if (node.secret_key === secretKey) {
        return node;
      }
    }
    return undefined;
  }
}

// Export a singleton instance
export const nodeRepository = new InMemoryNodeRepository();
