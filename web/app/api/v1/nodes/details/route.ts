import { nodeRepository } from "@/lib/repository/NodeRepository";
import { NodeService } from "@/lib/services/NodeService";
import { ApiResponse } from "@/lib/ApiResponse";
import { ErrorCode, ErrorMessage } from "@/constants/errors";

export const dynamic = "force-dynamic";

const nodeService = new NodeService(nodeRepository);

export async function GET(request: Request) {
  try {
    const authHeader = request.headers.get("authorization");
    if (authHeader !== `Token ${process.env.SECRET_KEY}`) {
      return new ApiResponse().failure(ErrorCode.INVALID_SECRET, "Unauthorized access", "authorization", 401);
    }

    const nodes = nodeService.getAllNodeDetails();

    return new ApiResponse().success({ nodes }, 200);
  } catch (error) {
    return new ApiResponse().failure(ErrorCode.INTERNAL_ERROR, ErrorMessage.INTERNAL_ERROR, null, 500);
  }
}
