import { nodeRepository } from "@/lib/repository/NodeRepository";
import { NodeService } from "@/lib/services/NodeService";
import { ApiResponse } from "@/lib/ApiResponse";
import { ErrorCode, ErrorMessage } from "@/constants/errors";

export const dynamic = "force-dynamic";

const nodeService = new NodeService(nodeRepository);

export async function POST(request: Request) {
  try {
    // Removed auth check to allow public dashboard viewing of telemetry

    const body = await request.json().catch(() => ({}));
    const page = parseInt(body.page || "1", 10);
    const size = parseInt(body.size || "10", 10);
    const sortBy = body.sortBy || "registered_at";
    const sortOrder = body.sortOrder || "desc";

    let nodes = nodeService.getAllNodeDetails();

    // Sorting
    nodes.sort((a: any, b: any) => {
      const valA = a[sortBy];
      const valB = b[sortBy];
      
      if (valA === valB) return 0;
      if (valA === undefined || valA === null) return 1;
      if (valB === undefined || valB === null) return -1;
      
      let comparison = 0;
      if (typeof valA === "string" && typeof valB === "string") {
        comparison = valA.localeCompare(valB);
      } else {
        comparison = valA < valB ? -1 : 1;
      }
      
      return sortOrder === "desc" ? -comparison : comparison;
    });

    // Pagination
    const total = nodes.length;
    const totalPages = Math.ceil(total / size);
    const startIndex = (page - 1) * size;
    const paginatedNodes = nodes.slice(startIndex, startIndex + size);

    return new ApiResponse().success({ 
      nodes: paginatedNodes,
      pagination: {
        total,
        page,
        size,
        totalPages
      }
    }, 200);
  } catch (error) {
    return new ApiResponse().failure(ErrorCode.INTERNAL_ERROR, ErrorMessage.INTERNAL_ERROR, null, 500);
  }
}
