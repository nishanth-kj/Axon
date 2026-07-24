import DocsLayout from "@/components/layout/DocsLayout";
import { CodeBlock } from "@/components/ui/CodeBlock";
import docsData from "@/lib/data/docs.json";

export default function DocsPage() {
  return (
    <DocsLayout title="Documentation">
      <div className="space-y-16">
        {docsData.map((section) => (
          <section key={section.id} id={section.id} className="scroll-mt-32">
            <h2 className="text-2xl font-bold text-foreground mb-4">{section.title}</h2>
            <p className="text-muted-foreground leading-relaxed mb-4">
              {section.content}
            </p>

            {section.code && (
              <CodeBlock code={section.code} language={section.language} />
            )}

            {section.endpoints && (
              <div className="space-y-10 mt-8">
                {section.endpoints.map((ep) => (
                  <div key={ep.id} id={ep.id} className="border border-border rounded-xl p-6 bg-card scroll-mt-32">
                    <h3 className="text-lg font-semibold flex items-center gap-3 mb-4">
                      <span className={`px-2 py-1 rounded text-xs font-bold uppercase tracking-wider ${
                        ep.method === 'GET' ? 'bg-green-500/10 text-green-500' : 
                        ep.method === 'POST' ? 'bg-primary/10 text-primary' : 
                        'bg-muted text-muted-foreground'
                      }`}>
                        {ep.method}
                      </span>
                      {ep.path}
                    </h3>
                    <p className="text-muted-foreground mb-4 text-sm">
                      {ep.description}
                    </p>
                    
                    {ep.payloadTitle && (
                      <h4 className="text-sm font-semibold mb-2">{ep.payloadTitle}</h4>
                    )}
                    {ep.code && (
                      <CodeBlock code={ep.code} language={ep.language} />
                    )}
                  </div>
                ))}
              </div>
            )}
          </section>
        ))}
      </div>
    </DocsLayout>
  );
}
