export enum ErrorCode {
  INVALID_PAYLOAD = 400,
  INVALID_STATUS = 400,
  MISSING_FIELDS = 400,
  NODE_NOT_FOUND = 404,
  IP_MISMATCH = 403,
  INVALID_SECRET = 403,
  UNREGISTERED_NODE = 401,
  INTERNAL_ERROR = 500,
}

export enum ErrorMessage {
  INVALID_PAYLOAD = "Invalid request payload",
  INVALID_STATUS = "Invalid status value",
  MISSING_FIELDS = "Missing node_id or secretKey",
  NODE_NOT_FOUND = "Node not found in registry",
  IP_MISMATCH = "IP mismatch",
  INVALID_SECRET = "Invalid secret key",
  UNREGISTERED_NODE = "Node is not registered or session expired",
  INTERNAL_ERROR = "Internal server error",
}
