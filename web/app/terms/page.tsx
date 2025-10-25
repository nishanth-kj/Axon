import SubpageLayout from "@/components/layout/SubpageLayout";

export default function TermsPage() {
  return (
    <SubpageLayout title="Terms of Service">
      <section className="space-y-8">
        <p>
          By using Axon, you agree to the following terms and conditions.
        </p>
        <div>
          <h2 className="text-xl font-bold text-foreground mb-4">Acceptable Use</h2>
          <p>
            You agree to use Axon for legal purposes only. We do not monitor your traffic, but we expect users to respect global laws and regulations.
          </p>
        </div>
        <div>
          <h2 className="text-xl font-bold text-foreground mb-4">License</h2>
          <p>
            Axon is released under the GPLv3 license. You are free to use, modify, and distribute the software according to the license terms.
          </p>
        </div>
      </section>
    </SubpageLayout>
  );
}
