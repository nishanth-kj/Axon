import { nodeRepository } from "@/lib/repository/NodeRepository";
import { logger } from "@/lib/utils/logger";

export async function register() {
  if (process.env.NEXT_RUNTIME === "nodejs") {
    logger.log("Starting Proxy Registry Health Checker...");
    const crypto = require("crypto");

    // Avoid multiple intervals in dev mode hot-reloading
    if (!globalThis.healthCheckInterval) {
      globalThis.healthCheckInterval = setInterval(async () => {
        const nodes = nodeRepository.getAll();
        const nowMs = Date.now();

        for (const node of nodes) {
          const node_id = node.node_id;
          
          try {
            const healthUrl = `http://${node.ip}:${node.verification_port}/api/v1/health`;
            
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 3000);

            const response = await fetch(healthUrl, {
              method: "GET",
              signal: controller.signal,
            }).finally(() => clearTimeout(timeoutId));

            if (response.ok) {
              node.last_seen = new Date().toISOString();
              node.failure_count = 0;
              nodeRepository.save(node);
              continue; // Health check passed!
            } else {
              throw new Error("Health check returned non-200");
            }
          } catch (error) {
            // Node failed to respond
            node.failure_count += 1;
            logger.warn(`Node ${node_id} health check failed. Failures: ${node.failure_count}`);
            
            if (node.failure_count >= 3) {
              logger.log(`Node ${node_id} removed from registry after 3 failures.`);
              nodeRepository.delete(node_id);
            } else {
              nodeRepository.save(node);
            }
          }
        }
      }, 10000); // 10 seconds
    }
  }
}

// Add global declaration for the interval
declare global {
  var healthCheckInterval: NodeJS.Timeout | undefined;
}
