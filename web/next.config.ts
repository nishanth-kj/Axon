import type { NextConfig } from "next";

const isGithubActions = process.env.GITHUB_ACTIONS || false;
let basePath = "";
if (isGithubActions && process.env.GITHUB_REPOSITORY) {
  const repoName = process.env.GITHUB_REPOSITORY.replace(/.*?\//, "");
  basePath = `/${repoName}`;
}

const nextConfig: NextConfig = {
  output: process.env.STATIC_EXPORT === "true" ? "export" : undefined,
  pageExtensions: process.env.STATIC_EXPORT === "true" 
    ? ["tsx", "jsx"] 
    : ["ts", "tsx", "js", "jsx"],
  images: {
    unoptimized: true,
  },
  basePath: basePath || "",
};

export default nextConfig;
