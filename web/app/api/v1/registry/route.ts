import { nodeRepository } from "@/lib/repository/NodeRepository";
import { NodeService } from "@/lib/services/NodeService";
import { ApiResponse } from "@/lib/ApiResponse";
import { ErrorCode, ErrorMessage } from "@/constants/errors";
import { SuccessMessage } from "@/constants/success";
import { getClientIp } from "@/lib/utils/ip";

import crypto from "crypto";
// Instantiate the service with the repository
const nodeService = new NodeService(nodeRepository);

export async function POST(request: Request) {
  try {
    const authHeader = request.headers.get("authorization");
    if (authHeader !== `Token ${process.env.SECRET_KEY}`) {
      return new ApiResponse().failure(ErrorCode.INVALID_SECRET, "Unauthorized registry access", "authorization", 401);
    }

    const body = await request.json();
    let { status, node_id, port } = body;

    // Securely extract the IP from the request headers
    const ip = getClientIp(request);

    if (typeof status !== "number" || !ip || ip === "unknown" || !port) {
      return new ApiResponse().failure(ErrorCode.INVALID_PAYLOAD, ErrorMessage.INVALID_PAYLOAD, null);
    }

    // Unregister
    if (status === 0) {
      if (!node_id) {
        return new ApiResponse().failure(ErrorCode.INVALID_PAYLOAD, "node_id is required for unregistering", "node_id", 400);
      }
      nodeService.unregisterNode(node_id);
      return new ApiResponse().success(SuccessMessage.UNREGISTERED_SUCCESSFULLY, 200);
    }

    // Register
    if (status === 1) {
      const generatedNodeId = node_id || crypto.randomUUID();
      await nodeService.registerNode(generatedNodeId, ip, port);
      return new ApiResponse().success({ message: SuccessMessage.REGISTRATION_PENDING, node_id: generatedNodeId }, 202);
    }

    return new ApiResponse().failure(ErrorCode.INVALID_STATUS, ErrorMessage.INVALID_STATUS, "status", 400);
  } catch (error) {
    return new ApiResponse().failure(ErrorCode.INTERNAL_ERROR, ErrorMessage.INTERNAL_ERROR, null, 500);
  }
}
