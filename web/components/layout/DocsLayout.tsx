import { Navbar } from "@/components/Navbar";
import { Footer } from "@/components/Footer";
import docsData from "@/lib/data/docs.json";

export default function DocsLayout({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col min-h-screen bg-background">
      <Navbar />
      <div className="flex-1 max-w-7xl mx-auto w-full pt-32 pb-24 px-6 md:px-8 flex flex-col md:flex-row gap-12">
        
        {/* Left Sidebar */}
        <aside className="w-full md:w-64 flex-shrink-0">
          <div className="sticky top-32 space-y-8">
            <div>
              <h3 className="font-semibold text-foreground mb-4">Contents</h3>
              <ul className="space-y-3">
                {docsData.map((section) => (
                  <li key={section.id}>
                    <a 
                      href={`#${section.id}`} 
                      className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                    >
                      {section.title}
                    </a>
                    {section.endpoints && (
                      <ul className="mt-2 ml-4 border-l border-border pl-4 space-y-2">
                        {section.endpoints.map((ep) => (
                          <li key={ep.id}>
                            <a 
                              href={`#${ep.id}`}
                              className="text-xs text-muted-foreground hover:text-foreground transition-colors"
                            >
                              {ep.title}
                            </a>
                          </li>
                        ))}
                      </ul>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 max-w-3xl">
          <h1 className="text-4xl md:text-5xl font-heading font-bold mb-12 tracking-tight">
            {title}
          </h1>
          <div className="prose prose-neutral dark:prose-invert max-w-none font-sans text-muted-foreground leading-relaxed">
            {children}
          </div>
        </main>
      </div>
      <Footer />
    </div>
  );
}
