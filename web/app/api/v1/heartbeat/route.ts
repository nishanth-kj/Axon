import { ApiResponse } from "@/lib/ApiResponse";
import { ErrorCode, ErrorMessage } from "@/constants/errors";
import { SuccessMessage } from "@/constants/success";
import { getClientIp, isLocalIp } from "@/lib/utils/ip";
import { nodeRepository } from "@/lib/repository/NodeRepository";
import { NodeService } from "@/lib/services/NodeService";

export const dynamic = "force-dynamic";

const nodeService = new NodeService(nodeRepository);

export async function GET() {
  return new ApiResponse().success({
    status: "alive",
    timestamp: new Date().toISOString(),
  }, 200);
}

export async function POST(request: Request) {
  try {
    const authHeader = request.headers.get("authorization");
    const secretKey = authHeader?.startsWith("Token ") ? authHeader.substring(6) : null;

    if (!secretKey) {
      return new ApiResponse().failure(ErrorCode.MISSING_FIELDS, "Missing or invalid Token in Authorization header", "authorization", 401);
    }

    const clientIp = getClientIp(request);
    const isClientLocal = isLocalIp(clientIp);

    const result = nodeService.verifyHeartbeat(secretKey, clientIp, isClientLocal);

    if (!result.success) {
      let errorCode = ErrorCode.INTERNAL_ERROR;
      let errorMessage = ErrorMessage.INTERNAL_ERROR;
      
      if (result.code === 404) {
        errorCode = ErrorCode.NODE_NOT_FOUND;
        errorMessage = ErrorMessage.NODE_NOT_FOUND;
      } else if (result.code === 403) {
        errorCode = ErrorCode.IP_MISMATCH;
        errorMessage = ErrorMessage.IP_MISMATCH;
      }

      return new ApiResponse().failure(errorCode, errorMessage, result.field, result.code || 500);
    }

    return new ApiResponse().success(SuccessMessage.PROXY_VERIFIED, 200);
  } catch (error) {
    return new ApiResponse().failure(ErrorCode.INTERNAL_ERROR, ErrorMessage.INTERNAL_ERROR, null, 500);
  }
}
