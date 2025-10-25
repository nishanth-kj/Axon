import { Hero } from "@/components/sections/Hero";
import { Features } from "@/components/sections/Features";
import { Architecture } from "@/components/sections/Architecture";
import { OpenSource } from "@/components/sections/OpenSource";
import { Platforms } from "@/components/sections/Platforms";
import { Footer } from "@/components/Footer";
import { Separator } from "@/components/ui/separator";

import { Navbar } from "@/components/Navbar";


export default function Home() {
  return (
    <div className="flex flex-col min-h-screen bg-background selection:bg-foreground selection:text-background">
      <Navbar />



      <main className="flex-1">

        <Hero />
        <Separator className="container max-w-5xl mx-auto opacity-50" />
        <Features />
        <Separator className="container max-w-5xl mx-auto opacity-50" />
        <Architecture />
        <Separator className="container max-w-5xl mx-auto opacity-50" />
        <OpenSource />
        <Platforms />
      </main>

      <Footer />
    </div>
  );
}
