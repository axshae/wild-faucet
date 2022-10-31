import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { WildFaucet } from "../target/types/wild_faucet";

describe("wild-faucet", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.WildFaucet as Program<WildFaucet>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
