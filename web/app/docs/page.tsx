import SubpageLayout from "@/components/layout/SubpageLayout";

export default function DocsPage() {
  return (
    <SubpageLayout title="Documentation">
      <section className="space-y-8">
        <div>
          <h2 className="text-xl font-bold text-foreground mb-4">Introduction</h2>
          <p>
            Welcome to the Axon documentation. Axon is a decentralized privacy infrastructure designed to provide secure, anonymous networking for everyone.
          </p>
        </div>
        <div>
          <h2 className="text-xl font-bold text-foreground mb-4">Getting Started</h2>
          <p>
            To start using Axon, you can download the client for your platform or use the CLI for advanced configuration.
          </p>
          <pre className="bg-muted p-4 rounded-lg mt-4 font-mono text-xs">
            $ axon init
          </pre>
        </div>
      </section>
    </SubpageLayout>
  );
}
