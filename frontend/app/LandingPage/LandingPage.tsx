import React from "react";
import Hero from "../sections/Hero/Hero";
import SavingsProducts from "../components/SavingsProducts";
import FAQ from "../components/FAQ";
import Newsletter from "../components/Newsletter";

const LandingPage: React.FC = () => {
  return (
    <main>
      <Hero
        headline={["Save Smarter.", "Grow", "Together.", "On-Chain."]}
        subheadline="Decentralized savings powered by Stellar & Soroban. Secure, transparent, and built for your financial future."
        primaryCta={{ label: "Start Saving", href: "#start-saving" }}
        secondaryCta={{ label: "Explore Pools", href: "#explore-pools" }}
        imageSrc="/hero.png"
        imageAlt="Glowing crypto vault with gold coins"
        stat={{ label: "Annual Yield", value: "12% APY" }}
      />

      <SavingsProducts />
      <FAQ />
      <Newsletter />
    </main>
  );
};

export default LandingPage;

