import type { NextConfig } from "next";

const isGithubActions = process.env.GITHUB_ACTIONS || false;
let repo = "";

if (isGithubActions && process.env.GITHUB_REPOSITORY) {
  repo = process.env.GITHUB_REPOSITORY.replace(/.*?\//, "");
}

const nextConfig: NextConfig = {
  output: process.env.STATIC_EXPORT === "true" ? "export" : undefined,
  pageExtensions: process.env.STATIC_EXPORT === "true" 
    ? ["tsx", "jsx"] 
    : ["ts", "tsx", "js", "jsx"],
  images: {
    unoptimized: true,
  },
  basePath: repo ? `/${repo}` : "",
};

export default nextConfig;
