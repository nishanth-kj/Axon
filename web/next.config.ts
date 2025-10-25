import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  images: {
    unoptimized: true,
  },
  // Note: If deploying to https://<username>.github.io/<repo-name>/, uncomment the next line
  // basePath: "/Axon",
};

export default nextConfig;
