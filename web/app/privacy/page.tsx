import SubpageLayout from "@/components/layout/SubpageLayout";

export default function PrivacyPage() {
  return (
    <SubpageLayout title="Privacy Policy">
      <section className="space-y-8">
        <p>
          At Axon, your privacy is our core mission. We do not collect, store, or sell any personal data.
        </p>
        <div>
          <h2 className="text-xl font-bold text-foreground mb-4">Zero Logging</h2>
          <p>
            The Axon network is designed to be zero-knowledge. We do not log IP addresses, traffic data, or any identifiable information.
          </p>
        </div>
        <div>
          <h2 className="text-xl font-bold text-foreground mb-4">Encryption</h2>
          <p>
            All data routed through the Axon network is protected by multi-layered, end-to-end encryption.
          </p>
        </div>
      </section>
    </SubpageLayout>
  );
}
