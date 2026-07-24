import { ApiResponse } from "@/lib/ApiResponse";
import { nodeRepository } from "@/lib/repository/NodeRepository";
import { NodeService } from "@/lib/services/NodeService";

export const dynamic = "force-dynamic";

const nodeService = new NodeService(nodeRepository);

export async function GET() {
  const nodes = nodeService.getActiveNodes();
  return new ApiResponse().success({ nodes }, 200);
}
