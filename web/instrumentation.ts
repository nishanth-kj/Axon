import { nodeRepository } from "@/lib/repository/NodeRepository";
import { logger } from "@/lib/utils/logger";

export async function register() {
  if (process.env.NEXT_RUNTIME === "nodejs") {
    logger.log("Starting Proxy Registry Health Checker...");
    const crypto = require("crypto");

    // Avoid multiple intervals in dev mode hot-reloading
    if (!globalThis.healthCheckInterval) {
      globalThis.healthCheckInterval = setInterval(() => {
        const nodes = nodeRepository.getAll();
        const nowMs = Date.now();

        for (const node of nodes) {
          const node_id = node.node_id;
          // Check if the proxy missed the last challenge (more than 15s since last seen)
          const lastSeenMs = new Date(node.last_seen).getTime();
          
          if (nowMs - lastSeenMs > 15000) {
            node.failure_count += 1;
            logger.warn(`Node ${node_id} failed to callback in time. Failures: ${node.failure_count}`);
            
            if (node.failure_count >= 3) {
              logger.log(`Node ${node_id} removed from registry after 3 failures.`);
              nodeRepository.delete(node_id);
              continue;
            }
          }

          // Generate the deterministic secret and challenge the proxy
          const masterSecret = process.env.SECRET_KEY || 'default_secret';
          const newSecret = crypto.createHmac('sha256', masterSecret).update(node_id).digest('hex');
          node.secret_key = newSecret;
          nodeRepository.save(node);

          try {
            const verifyUrl = `http://${node.ip}:${node.port}/api/v1/verify`;
            
            fetch(verifyUrl, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ secretKey: newSecret }),
            }).catch(() => {
              // Fire and forget; if the proxy is down, it won't callback and failure_count will increase next tick
            });
          } catch (error) {
            // Ignore fetch errors
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
