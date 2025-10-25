"use client";

import { useEffect, useRef } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Shield,
  Zap,
  Lock,
  Globe,
  Share2,
  MonitorSmartphone,
} from "lucide-react";

gsap.registerPlugin(ScrollTrigger);

const features = [
  {
    title: "Onion Routing",
    description:
      "Multi-layered encryption ensures that no single node in the network knows both the source and destination.",
    icon: Shield,
  },
  {
    title: "Relay Mesh",
    description:
      "A resilient, peer-to-peer network of community-run nodes providing censorship resistance.",
    icon: Share2,
  },
  {
    title: "Encrypted Tunnels",
    description:
      "Military-grade AES-256-GCM encryption for every connection, protecting your data from surveillance.",
    icon: Lock,
  },
  {
    title: "DHT Discovery",
    description:
      "Serverless peer discovery using Distributed Hash Tables, making the network impossible to block.",
    icon: Globe,
  },
  {
    title: "Multi-hop Routing",
    description:
      "Dynamic routing through multiple geographic locations to obfuscate your digital footprint.",
    icon: Zap,
  },
  {
    title: "Cross-platform",
    description:
      "Native support for Linux, macOS, Windows, iOS, and Android, plus a powerful CLI for power users.",
    icon: MonitorSmartphone,
  },
];

export const Features = () => {
  const sectionRef = useRef<HTMLElement>(null);
  const cardsRef = useRef<HTMLDivElement[]>([]);

  useEffect(() => {
    gsap.fromTo(
      cardsRef.current,
      { y: 40, opacity: 0 },
      {
        y: 0,
        opacity: 1,
        duration: 0.8,
        stagger: 0.1,
        ease: "power2.out",
        scrollTrigger: {
          trigger: sectionRef.current,
          start: "top 80%",
        },
      }
    );
  }, []);

  return (
    <section id="features" ref={sectionRef} className="py-24 px-4 bg-muted/20">

      <div className="container max-w-6xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-3xl md:text-5xl font-heading font-bold mb-4">
            Security by Design
          </h2>
          <p className="text-muted-foreground max-w-2xl mx-auto font-sans">
            Axon combines cutting-edge cryptography with decentralized networking
            to provide a premium privacy shield for your digital life.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <Card
              key={index}
              ref={(el) => { if (el) cardsRef.current[index] = el; }}
              className="group bg-card border-border hover:shadow-md transition-all duration-300"
            >
              <CardHeader>
                <div className="p-2.5 rounded-md bg-muted w-fit mb-4 text-foreground transition-colors group-hover:bg-foreground group-hover:text-background">
                  <feature.icon className="h-5 w-5" />
                </div>
                <CardTitle className="text-lg font-heading">
                  {feature.title}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription className="text-muted-foreground text-sm leading-relaxed">
                  {feature.description}
                </CardDescription>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
};
