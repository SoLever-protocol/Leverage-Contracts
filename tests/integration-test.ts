import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Solever } from "../target/types/solever";
// import { Metaplex } from "@metaplex-foundation/js";
import { fetchDigitalAsset, findMetadataPda, createV1, TokenStandard, mintV1, mplTokenMetadata} from "@metaplex-foundation/mpl-token-metadata";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { generateSigner, publicKey, signerIdentity } from "@metaplex-foundation/umi";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import BN = require("@coral-xyz/anchor");
import * as splToken from "@solana/spl-token";
import web3, { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { base64 } from "@metaplex-foundation/umi/serializers";


describe("test_lrt", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const umi = createUmi(anchor.getProvider().connection)

  const program = anchor.workspace.TestLRT as Program<TestLRT>;
  // const token_program = anchor.workspace.Token as Program<Token>;

  it("Creates mint", async () => {
    // find the address of the mint account
    const [evSOLMintPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("evSOL")],
      program.programId,
    );

    // umi.eddsa.findPda(program.programId, [Buffer.from("evSOL")])
    const metadata = {
        uri: "https://raw.githubusercontent.com/solana-developers/program-examples/new-examples/tokens/tokens/.assets/spl-token.json",
        name: "Solever evSOL",
        symbol: "EVSOL",
      };
    console.log("running createMint")
    //console.log("metadata account: " + (findMetadataPda(umi, {mint: publicKey(evSOLMintPDA)})))

    const [metadata_PDA_key, metadata_bump] = findMetadataPda(umi, {mint: publicKey(evSOLMintPDA)});
    //const metadataAccountString = bs58.encode(((new BN.BN(bs58.decode(metadata_PDA_key))).add(new BN.BN(metadata_bump))).toArray());
    const metadataAccountString = bs58.encode(((new BN.BN(bs58.decode(metadata_PDA_key)))).toArray());
    console.log("metadata account: " + metadataAccountString);

    // get the PDA for the new collateral tracker
    const [collateralTrackerPDA, _] = await PublicKey.findProgramAddressSync([anchor.utils.bytes.utf8.encode('evSOL'),
      anchor.utils.bytes.utf8.encode('slashing')
    ], program.programId)

    const tx = await program.methods.createMint(metadata.uri, metadata.name, metadata.symbol)
    .accounts(
      {
        collateralTracker: collateralTrackerPDA,
        evsolMint: evSOLMintPDA,
        //metadataAccount: (await fetchDigitalAsset(umi, publicKey(evSOLMintPDA))).metadata.publicKey
        // metadataAccount: (await fetchDigitalAsset(umi, evSOLMintPDA)).metadata.publicKey
        //metadataAccount: new anchor.web3.PublicKey(findMetadataPda(umi, {mint: publicKey(evSOLMintPDA)}))
        //metadataAccount: anchor.web3.PublicKey.findProgramAddressSync()
        metadataAccount: metadataAccountString
      }
    )
    .rpc();
    console.log("Your transaction signature", tx);
    console.log("Token Mint: ", evSOLMintPDA.toString());
  });
  it("Deposits fungible token", async () => {
    // ===== create the test token =====
    // decided against the use of metaplex overall--commenting this stuff out and using splToken
    // ===== begin block comment ===============================
      /*
      generate the signer for the mint account
      const mint = generateSigner(umi);
      const user = generateSigner(umi);

      //umi.use(signerIdentity(user))

      //context.programs.add("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")
      
      umi.use(mplTokenMetadata())
      // create the actual token
      const result = createV1(umi, {
        // linter gives an error, but 'asset' should be a field of CreateV1InstructionAccounts
        mint,
        name: 'Test Fungible Token',
        //tokenStandard: TokenStandard.NonFungible,
        tokenStandard: TokenStandard.Fungible,
        //sellerFeeBasisPoints: 0
    }).sendAndConfirm(umi);

    // mint the token to the test user
    await mintV1(umi, {
    mint: mint.publicKey,
    amount: 1,
    tokenOwner: user.publicKey,
    tokenStandard: TokenStandard.NonFungible,
  }).sendAndConfirm(umi);
  */
  // ===== end block comment ===============================
  // const wallet = anchor.web3.Keypair.generate();
  // console.log("wallet: " + wallet)

  // suspected issue: anchor.getProvider().connection returns a different kind of object than the @solana/web3.Connection
  // and anchor.getProvider().wallet returns a different kind of object than the @solana/web3.
  // to address this, there are two approaches:
  //    A) cast from one type to the other
  //    B) just completely use the @solana/ library instead of the @coral-xyz/ library

  // this is also goofy, but create a minter to create the mint and mint the tokens to the coral-xyz/anchor account
  // which actually makes the test deposit
  const minter_wallet = web3.Keypair.generate();
  //const connection = new web3.Connection(anchor.getProvider().connection.rpcEndpoint);

  //console.log(await connection.requestAirdrop(minter_wallet.publicKey, 2));
  const airdrop_transaction = await anchor.getProvider().connection.requestAirdrop(minter_wallet.publicKey, 2*LAMPORTS_PER_SOL);
  //const airdrop_transaction = await connection.requestAirdrop(minter_wallet.publicKey, 2*LAMPORTS_PER_SOL);
  console.log("Completed airdrop, transaction is " + airdrop_transaction);
  console.log(await anchor.getProvider().connection.confirmTransaction(airdrop_transaction));
  console.log("Balance is: " + await anchor.getProvider().connection.getBalance(minter_wallet.publicKey));

  const test_token_mint = await splToken?.createMint(
    anchor.getProvider().connection,
    minter_wallet, 
    minter_wallet.publicKey,
    null, 9, undefined, {}
  );

  // create the associated account for the Anchor wallet to hold the minted token
  const test_token_associated_acct = await splToken.createAssociatedTokenAccount(
    anchor.getProvider().connection,
    minter_wallet,
    test_token_mint,
    anchor.getProvider().publicKey
  );

  // mint test token to our anchor provider public key
  const mint_tx = await splToken.mintTo(
    anchor.getProvider().connection,
    minter_wallet,
    test_token_mint,
    test_token_associated_acct,
    minter_wallet.publicKey,
    10000
  );

  
  // initialize our contracts
  const [evSOLMintPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("evSOL")],
      program.programId,
    );
// seems like we don't need to re-initialize anything

    // create our evsol associated account
    const evsol_associated_acct = splToken.getAssociatedTokenAddressSync(evSOLMintPDA, anchor.getProvider().publicKey);

    // get the PDA for the new collateral tracker
    const [collateralTrackerPDA, _1] = await PublicKey.findProgramAddressSync([anchor.utils.bytes.utf8.encode('evSOL'),
      anchor.utils.bytes.utf8.encode('slashing')
    ], program.programId)

    const [holdingsPDA, _2] = await PublicKey.findProgramAddressSync([anchor.utils.bytes.utf8.encode('evSOL'),
      anchor.utils.bytes.utf8.encode('holdings'), test_token_mint.toBuffer()
    ], program.programId)

    console.log("running deposit transaction")
    // deposit
    const deposit_tx = await program.methods.deposit(new anchor.BN(50)).accounts({
// TODO
      mintTo: evsol_associated_acct,
      depositFrom: test_token_associated_acct,
      depositorSigner: anchor.getProvider().publicKey,
      // TODO: enforce depositing to our account, for now can be any
      // and, for now, depositing back to the same account we are depositing from
      //depositTo: test_token_associated_acct,
      depositTo: holdingsPDA,
      collateralTracker: collateralTrackerPDA,
      evsolMint: evSOLMintPDA,
      depositMint: test_token_mint.toBuffer()
    }).rpc();
    //expect(bs58.decode(await (program.methods.tokensDeposited().accounts({
      //collateralTracker: collateralTrackerPDA,
    //}).rpc()))).to.equal(new anchor.BN(50).toBuffer())
  });
  it("slashes", async () => {
    const [collateralTrackerPDA, _] = await PublicKey.findProgramAddressSync([anchor.utils.bytes.utf8.encode('evSOL'),
      anchor.utils.bytes.utf8.encode('slashing')
    ], program.programId)

    await program.methods.slash(new anchor.BN(5)).accounts({
      collateralTrackerPDA: collateralTrackerPDA
    }).rpc()
  })
  
});
