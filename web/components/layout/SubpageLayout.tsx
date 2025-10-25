import { Navbar } from "@/components/Navbar";
import { Footer } from "@/components/Footer";

export default function SubpageLayout({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col min-h-screen bg-background">
      <Navbar />
      <main className="flex-1 pt-32 pb-24 px-6">
        <div className="container max-w-3xl mx-auto">
          <h1 className="text-4xl md:text-5xl font-heading font-bold mb-12 tracking-tight text-center">
            {title}
          </h1>
          <div className="prose prose-neutral dark:prose-invert max-w-none font-sans text-muted-foreground leading-relaxed">
            {children}
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
}
