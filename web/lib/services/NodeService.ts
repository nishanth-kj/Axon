import { INodeRepository, ProxyNode } from "@/lib/repository/NodeRepository";
import crypto from "crypto";
import { logger } from "@/lib/utils/logger";

export class NodeService {
  constructor(private repository: INodeRepository) {}

  /**
   * Registers a new node with PENDING status and attempts to verify it.
   */
  async registerNode(node_id: string, ip: string, port: number, verification_port?: number): Promise<void> {
    // Evict any existing nodes sharing this physical endpoint (IP:Port) to prevent duplicates
    const existingNodes = this.repository.getAll().filter(n => n.ip === ip && n.port === port);
    for (const oldNode of existingNodes) {
      logger.log(`Evicting old node ${oldNode.node_id} at ${ip}:${port} due to new registration`);
      this.repository.delete(oldNode.node_id);
    }

    // Deterministically generate the node's secret key using the master SECRET_KEY
    const masterSecret = process.env.SECRET_KEY || 'default_secret';
    const secretKey = crypto.createHmac('sha256', masterSecret).update(node_id).digest('hex');
    logger.log(`[TESTING] Generated deterministic secretKey for ${node_id}: ${secretKey}`);
    const now = new Date().toISOString();

    const node: ProxyNode = {
      node_id,
      ip,
      port,
      verification_port: verification_port || port,
      status: "PENDING",
      secret_key: secretKey,
      last_seen: now,
      registered_at: now,
      failure_count: 0,
    };

    this.repository.save(node);

    // Send the secret key to the proxy
    this.sendVerificationRequest(ip, verification_port || port, secretKey);
  }

  /**
   * Unregisters a node by ID.
   */
  unregisterNode(node_id: string): void {
    this.repository.delete(node_id);
  }

  /**
   * Verifies a heartbeat request. If valid, marks the node as ACTIVE.
   */
  verifyHeartbeat(secretKey: string, clientIp: string, isClientLocal: boolean): { success: boolean; error?: string; code?: number; field?: string } {
    const node = this.repository.findBySecretKey(secretKey);

    if (!node) {
      logger.log(`[TESTING] Heartbeat 401! Received secretKey: ${secretKey}`);
      return { success: false, error: "Node not found for this secretKey", code: 401, field: "secretKey" };
    }

    const isNodeLocal = this.isLocalIp(node.ip);
    
    // Strict IP matching with local dev forgiveness
    if (!(isNodeLocal && isClientLocal) && clientIp !== node.ip) {
      logger.error(`[AUTH FAILED] IP Mismatch! Node IP: ${node.ip}, Client IP: ${clientIp}`);
      return { success: false, error: "IP Mismatch", code: 403, field: "ip" };
    }

    // Verification successful
    node.status = "ACTIVE";
    node.last_seen = new Date().toISOString();
    node.failure_count = 0;
    
    // Save updated node
    this.repository.save(node);

    return { success: true };
  }

  /**
   * Returns a list of active nodes for public routing.
   */
  getActiveNodes(): { node_id: string; ip: string; port: number; status?: string }[] {
    const isDebug = process.env.DEBUG === "true";
    return this.repository.getAll()
      .filter((node) => node.status === "ACTIVE" || (isDebug && node.status === "PENDING"))
      .map((node) => ({
        node_id: node.node_id,
        ip: node.ip,
        port: node.port,
        status: isDebug ? node.status : undefined,
      }));
  }

  /**
   * Returns all nodes with full telemetry details (excluding secret_key) for the dashboard.
   */
  getAllNodeDetails(): Omit<ProxyNode, "secret_key">[] {
    return this.repository.getAll().map((node) => {
      const { secret_key, ...details } = node;
      return details;
    });
  }

  private isLocalIp(ip: string): boolean {
    return ip === "localhost" || 
           ip === "127.0.0.1" || 
           ip === "::1" || 
           ip.startsWith("192.168.") || 
           ip.startsWith("10.");
  }

  private sendVerificationRequest(ip: string, verification_port: number, secretKey: string): void {
    try {
      const formattedIp = ip.includes(':') ? `[${ip}]` : ip;
      const verifyUrl = `http://${formattedIp}:${verification_port}/api/v1/verify`;
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000);

      fetch(verifyUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ secretKey }),
        signal: controller.signal,
      }).catch(() => {
        // Fire and forget; if it fails, proxy won't be able to callback and will remain PENDING
      }).finally(() => {
        clearTimeout(timeoutId);
      });
    } catch (err) {
      // Ignore fetch dispatch errors
    }
  }
}
