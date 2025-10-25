"use client";

import { useEffect, useRef } from "react";
import gsap from "gsap";
import { Button } from "@/components/ui/button";
import { Terminal, Download } from "lucide-react";

const GithubIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
  </svg>
);

export const Hero = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const titleRef = useRef<HTMLHeadingElement>(null);
  const subtitleRef = useRef<HTMLParagraphElement>(null);
  const actionsRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const tl = gsap.timeline({ defaults: { ease: "power3.out" } });

    tl.fromTo(
      titleRef.current,
      { y: 30, opacity: 0 },
      { y: 0, opacity: 1, duration: 1 }
    )
      .fromTo(
        subtitleRef.current,
        { y: 20, opacity: 0 },
        { y: 0, opacity: 1, duration: 0.8 },
        "-=0.6"
      )
      .fromTo(
        actionsRef.current,
        { y: 15, opacity: 0 },
        { y: 0, opacity: 1, duration: 0.8 },
        "-=0.6"
      )
      .fromTo(
        terminalRef.current,
        { y: 30, opacity: 0 },
        { y: 0, opacity: 1, duration: 1 },
        "-=0.5"
      );
  }, []);

  return (
    <section
      ref={containerRef}
      className="relative min-h-[90vh] flex flex-col items-center justify-center pt-32 pb-24 px-4 overflow-hidden"
    >
      
      <div className="container max-w-6xl mx-auto text-center z-10">
        <h1
          ref={titleRef}
          className="text-6xl md:text-8xl font-heading font-bold tracking-tight mb-6"
        >
          Axon
        </h1>
        <p
          ref={subtitleRef}
          className="text-lg md:text-xl text-muted-foreground max-w-xl mx-auto mb-10 font-sans"
        >
          Decentralized Privacy Infrastructure. <br />
          Built for the next generation of secure networking.
        </p>

        <div
          ref={actionsRef}
          className="flex flex-wrap items-center justify-center gap-3 mb-20"
        >
          <Button
            size="lg"
            className="rounded-full px-8 h-12 text-sm font-medium transition-all hover:scale-[1.02] active:scale-95"
          >
            <Download className="mr-2 h-4 w-4" />
            Download Axon
          </Button>
          <Button
            variant="outline"
            size="lg"
            className="rounded-full px-8 h-12 text-sm font-medium border-border hover:bg-accent transition-all"
          >
            <GithubIcon className="mr-2 h-4 w-4" />
            Star on GitHub
          </Button>
        </div>

        {/* Minimal Terminal */}
        <div
          ref={terminalRef}
          className="max-w-3xl mx-auto rounded-lg border border-border bg-card shadow-sm overflow-hidden"
        >
          <div className="flex items-center gap-2 px-4 py-2 border-b border-border bg-muted/30">
            <div className="flex gap-1.5">
              <div className="w-2.5 h-2.5 rounded-full bg-border" />
              <div className="w-2.5 h-2.5 rounded-full bg-border" />
              <div className="w-2.5 h-2.5 rounded-full bg-border" />
            </div>
            <div className="flex items-center gap-2 ml-3 text-[10px] font-mono text-muted-foreground uppercase tracking-widest">
              <Terminal className="h-3 w-3" />
              <span>axon-daemon --init</span>
            </div>
          </div>
          <div className="p-8 text-left font-mono text-xs md:text-sm leading-relaxed">
            <div className="flex gap-3 mb-3">
              <span className="text-muted-foreground opacity-50">$</span>
              <span>axon init --network mainnet</span>
            </div>
            <div className="text-muted-foreground mb-1">[info] Initializing node...</div>
            <div className="text-muted-foreground mb-1">[info] Connecting to relay mesh...</div>
            <div className="text-foreground/80 mb-3">✔ Protocol handshake complete.</div>
            <div className="flex gap-3 mb-3">
              <span className="text-muted-foreground opacity-50">$</span>
              <span>axon status</span>
            </div>
            <div className="text-foreground font-medium">Node active. 128ms latency. Routing via 4 hops.</div>
            <div className="animate-pulse inline-block w-2 h-4 bg-foreground/30 ml-1 translate-y-1" />
          </div>
        </div>
      </div>
    </section>
  );
};
