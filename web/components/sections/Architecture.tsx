"use client";

import { useEffect, useRef } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { User, Server, Globe, ShieldCheck } from "lucide-react";

gsap.registerPlugin(ScrollTrigger);

export const Architecture = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const pathRef = useRef<SVGPathElement>(null);

  useEffect(() => {
    if (!pathRef.current) return;

    const path = pathRef.current;
    const length = path.getTotalLength();

    gsap.set(path, { strokeDasharray: length, strokeDashoffset: length });

    gsap.to(path, {
      strokeDashoffset: 0,
      duration: 2.5,
      repeat: -1,
      ease: "power1.inOut",
      scrollTrigger: {
        trigger: containerRef.current,
        start: "top 70%",
      },
    });
  }, []);

  return (
    <section id="architecture" className="py-32 px-4" ref={containerRef}>

      <div className="container max-w-6xl mx-auto">
        <div className="text-center mb-24">
          <h2 className="text-3xl md:text-5xl font-heading font-bold mb-4">
            Network Architecture
          </h2>
          <p className="text-muted-foreground max-w-2xl mx-auto">
            A sophisticated multi-hop relay system designed for maximum anonymity
            and minimal performance overhead.
          </p>
        </div>

        <div className="relative flex flex-col md:flex-row items-center justify-between gap-16 md:gap-4 max-w-5xl mx-auto py-12">
          {/* Node: User */}
          <div className="flex flex-col items-center gap-4 z-10">
            <div className="w-14 h-14 rounded-full bg-card border border-border flex items-center justify-center shadow-sm">
              <User className="h-6 w-6 text-foreground" />
            </div>
            <span className="text-xs font-medium uppercase tracking-widest text-muted-foreground">User</span>
          </div>

          {/* Node: Relay */}
          <div className="flex flex-col items-center gap-4 z-10">
            <div className="w-14 h-14 rounded-full bg-card border border-border flex items-center justify-center shadow-sm ring-4 ring-muted/30">
              <ShieldCheck className="h-6 w-6 text-foreground" />
            </div>
            <span className="text-xs font-medium uppercase tracking-widest text-muted-foreground text-center">Relay Nodes</span>
          </div>

          {/* Node: Exit */}
          <div className="flex flex-col items-center gap-4 z-10">
            <div className="w-14 h-14 rounded-full bg-card border border-border flex items-center justify-center shadow-sm">
              <Server className="h-6 w-6 text-foreground" />
            </div>
            <span className="text-xs font-medium uppercase tracking-widest text-muted-foreground text-center">Exit Nodes</span>
          </div>

          {/* Node: Internet */}
          <div className="flex flex-col items-center gap-4 z-10">
            <div className="w-14 h-14 rounded-full bg-foreground text-background flex items-center justify-center shadow-lg">
              <Globe className="h-6 w-6" />
            </div>
            <span className="text-xs font-medium uppercase tracking-widest text-foreground">Internet</span>
          </div>

          {/* Animated Path (SVG) */}
          <svg
            className="absolute top-1/2 left-0 w-full h-2 -translate-y-[28px] hidden md:block px-12"
            viewBox="0 0 800 2"
            fill="none"
          >
            <path
              ref={pathRef}
              d="M0,1 L800,1"
              stroke="var(--foreground)"
              strokeWidth="1.5"
              strokeDasharray="4 4"
              opacity="0.2"
            />
          </svg>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-12 mt-32">
          <div className="space-y-3">
            <h3 className="text-lg font-heading font-semibold">Zero-Knowledge</h3>
            <p className="text-muted-foreground text-sm leading-relaxed">
              Nodes only know the previous and next hop in the chain, never the entire path or your true identity.
            </p>
          </div>
          <div className="space-y-3">
            <h3 className="text-lg font-heading font-semibold">Quantum-Ready</h3>
            <p className="text-muted-foreground text-sm leading-relaxed">
              Future-proof encryption algorithms designed to withstand the era of quantum computing.
            </p>
          </div>
          <div className="space-y-3">
            <h3 className="text-lg font-heading font-semibold">Distributed P2P</h3>
            <p className="text-muted-foreground text-sm leading-relaxed">
              A self-healing network that automatically routes around failure points and network congestion.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
};
