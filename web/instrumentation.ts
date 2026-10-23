export async function register() {
  if (process.env.NEXT_RUNTIME === "nodejs") {
    console.log("Axon web is documentation and the node view. The registry API is the Actix server.");
  }
}
