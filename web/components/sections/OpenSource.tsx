"use client";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Code, Users, Scale, ShieldCheck } from "lucide-react";

const GithubIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
  </svg>
);

export const OpenSource = () => {
  return (
    <section className="py-24 px-4 bg-muted/10">
      <div className="container max-w-6xl mx-auto">
        <div className="border border-border rounded-2xl bg-card p-8 md:p-16 relative overflow-hidden">
          <div className="relative z-10 grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div>
              <Badge variant="outline" className="mb-6 font-mono text-[10px] uppercase tracking-widest px-3 py-1 bg-muted/50">
                GPLv3 LICENSED
              </Badge>
              <h2 className="text-3xl md:text-5xl font-heading font-bold mb-6 leading-tight">
                Open Code. <br />
                Universal Access.
              </h2>
              <p className="text-muted-foreground text-lg mb-8 font-sans leading-relaxed">
                Axon is committed to radical transparency. Our codebase is 100% open
                source and verified by the community. No backdoors, just code.
              </p>
              <div className="flex flex-wrap gap-4">
                <Button className="rounded-full px-8 h-12 text-sm font-medium">
                  <GithubIcon className="mr-2 h-4 w-4" />
                  View Source
                </Button>
                <Button variant="ghost" className="rounded-full px-8 h-12 text-sm font-medium">
                  Read Whitepaper
                </Button>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-px bg-border border border-border rounded-xl overflow-hidden shadow-sm">
              <div className="p-8 bg-card flex flex-col items-center text-center">
                <Users className="h-6 w-6 text-muted-foreground mb-4" />
                <h3 className="text-xl font-heading font-bold">500+</h3>
                <p className="text-xs text-muted-foreground uppercase tracking-widest mt-1">Contributors</p>
              </div>
              <div className="p-8 bg-card flex flex-col items-center text-center">
                <Scale className="h-6 w-6 text-muted-foreground mb-4" />
                <h3 className="text-xl font-heading font-bold">100%</h3>
                <p className="text-xs text-muted-foreground uppercase tracking-widest mt-1">Transparent</p>
              </div>
              <div className="p-8 bg-card flex flex-col items-center text-center">
                <ShieldCheck className="h-6 w-6 text-muted-foreground mb-4" />
                <h3 className="text-xl font-heading font-bold">Audited</h3>
                <p className="text-xs text-muted-foreground uppercase tracking-widest mt-1">Security</p>
              </div>
              <div className="p-8 bg-card flex flex-col items-center text-center">
                <Code className="h-6 w-6 text-muted-foreground mb-4" />
                <h3 className="text-xl font-heading font-bold">Copyleft</h3>
                <p className="text-xs text-muted-foreground uppercase tracking-widest mt-1">GPLv3</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
