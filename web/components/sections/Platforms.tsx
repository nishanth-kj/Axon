"use client";

import { Monitor, Smartphone, Terminal, Globe } from "lucide-react";


const platforms = [
  { name: "Desktop", icon: Monitor, status: "Stable", desc: "Windows, macOS, Linux" },
  { name: "Mobile", icon: Smartphone, status: "Stable", desc: "iOS, Android" },
  { name: "CLI", icon: Terminal, status: "LTS", desc: "For power users" },
  { name: "Web", icon: Globe, status: "Beta", desc: "Extension & Web App" },
];



export const Platforms = () => {
  return (
    <section id="platforms" className="py-24 px-4">

      <div className="container max-w-6xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-3xl md:text-5xl font-heading font-bold mb-4">
            Unified Access
          </h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            Axon is cross-platform by design. Secure every connection with a
            single decentralized identity.
          </p>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">


          {platforms.map((p, i) => (
            <div
              key={i}
              className="p-8 rounded-xl border border-border bg-card hover:bg-muted/30 transition-colors group flex flex-col items-center text-center shadow-sm"
            >
              <div className="w-12 h-12 rounded-lg bg-muted flex items-center justify-center mb-6 transition-all group-hover:scale-105">
                <p.icon className="h-5 w-5 text-foreground" />
              </div>
              <h3 className="text-lg font-heading font-semibold mb-2">{p.name}</h3>
              <p className="text-muted-foreground text-xs mb-6 h-8">{p.desc}</p>
              <span className={`text-[10px] font-mono uppercase tracking-widest px-2.5 py-1 rounded-full ${p.status === "Beta" ? "bg-muted text-foreground" : "bg-foreground text-background"}`}>
                {p.status}
              </span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
