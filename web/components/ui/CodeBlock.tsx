"use client";

import { useState } from "react";
import { Check, Copy } from "lucide-react";

interface CodeBlockProps {
  code: string;
  language?: string;
}

export function CodeBlock({ code, language }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy code:", err);
    }
  };

  return (
    <div className="relative group mt-4">
      {language && (
        <div className="absolute right-12 top-3 text-xs text-muted-foreground font-mono opacity-0 group-hover:opacity-100 transition-opacity">
          {language}
        </div>
      )}
      <button
        onClick={handleCopy}
        className="absolute right-3 top-3 p-2 rounded-md bg-muted/50 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors border border-transparent hover:border-border"
        aria-label="Copy code"
      >
        {copied ? <Check className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4" />}
      </button>
      <pre className="bg-muted/50 border border-border p-4 pt-12 rounded-lg font-mono text-xs text-foreground overflow-x-auto whitespace-pre-wrap">
        {code}
      </pre>
    </div>
  );
}
