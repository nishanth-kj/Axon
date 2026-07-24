/**
 * Extracts the real client IP address from request headers.
 * Prefers `x-forwarded-for` over `x-real-ip`, and falls back to a provided IP.
 * @param request The incoming HTTP Request object.
 * @param fallbackIp Optional fallback IP if headers are missing.
 * @returns The extracted IP string.
 */
export function getClientIp(request: Request, fallbackIp?: string): string {
  const forwardedFor = request.headers.get("x-forwarded-for");
  const realIp = request.headers.get("x-real-ip");
  
  let finalIp = fallbackIp || "unknown";
  
  if (forwardedFor) {
    finalIp = forwardedFor.split(',')[0].trim();
  } else if (realIp) {
    finalIp = realIp.trim();
  }

  // Clean up IPv4-mapped IPv6 addresses (e.g. ::ffff:127.0.0.1 -> 127.0.0.1)
  if (finalIp.startsWith("::ffff:")) {
    finalIp = finalIp.replace("::ffff:", "");
  }

  return finalIp;
}

/**
 * Checks if a given IP address belongs to a local development environment.
 * Matches localhost, IPv4 loopback, IPv6 loopback, and common private network prefixes.
 * @param ip The IP address string to check.
 * @returns boolean True if the IP is local.
 */
export function isLocalIp(ip: string): boolean {
  return ip === "localhost" || 
         ip === "127.0.0.1" || 
         ip === "::1" || 
         ip.startsWith("192.168.") || 
         ip.startsWith("10.");
}
