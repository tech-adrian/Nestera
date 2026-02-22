import React from 'react';
import './SavingsProducts.css';

interface ProductCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
}

const ProductCard: React.FC<ProductCardProps> = ({ icon, title, description }) => (
  <div className="savings-products__card">
    <div className="savings-productsIcon">
      {icon}
    </div>
    <div className="savings-products__card-text">
      <h3 className="savings-products__card-title">{title}</h3>
      <p className="savings-products__card-desc">{description}</p>
      <a href="#" className="savings-products__mockup-start" style={{ marginTop: '12px' }}>
        Learn More <span style={{ fontSize: '1.2rem' }}>→</span>
      </a>
    </div>
  </div>
);

const SavingsProducts: React.FC = () => {
  const products = [
    {
      title: 'Flexible Savings',
      description: 'Withdraw anytime. Best for emergency funds and daily liquidity. No lock periods.',
      icon: (
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <rect x="2" y="5" width="20" height="14" rx="2" ry="2"/>
          <line x1="2" y1="10" x2="22" y2="10"/>
        </svg>
      ),
    },
    {
      title: 'Locked Savings',
      description: 'Higher APY for fixed terms. Commit your funds and earn more with deterministic interest.',
      icon: (
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
      ),
    },
    {
      title: 'Goal-Based',
      description: 'Set a target. Auto-save until you reach your dream purchase or financial milestone.',
      icon: (
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="10"/>
          <circle cx="12" cy="12" r="6"/>
          <circle cx="12" cy="12" r="2"/>
        </svg>
      ),
    },
    {
      title: 'Group Savings',
      description: 'Save with friends and family. Pool funds for collective goals with shared transparent rules.',
      icon: (
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
          <circle cx="9" cy="7" r="4"/>
          <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
          <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
        </svg>
      ),
    },
  ];

  return (
    <section className="savings-products" id="savings-products">
      <div className="savings-products__container">
        <div className="savings-products__header">
          <h2 className="savings-products__title">
            Savings Products Tailored for You
          </h2>
          <p className="savings-products__description">
            Whether you want flexibility or higher returns, we have a pool for that.
          </p>
        </div>

        <div className="savings-products__content">
          {/* MOCKUP */}
          <div className="savings-products__mockup">
            <img 
              src="/mockup.png" 
              alt="Nestera Mobile App Mockup" 
              className="savings-products__mockup-img"
            />
            <div className="savings-products__mockup-overlay" />
            <div className="savings-products__mockup-content">
              <div className="savings-products__mockup-info">
                <span className="savings-products__mockup-category">Flexible Savings</span>
                <h3 className="savings-products__mockup-title">Start Saving Today</h3>
                <p className="savings-products__mockup-desc">
                  Join thousands of users growing their wealth securely on Stellar.
                </p>
              </div>
              <a href="#" className="savings-products__mockup-start">
                Start <span style={{ fontSize: '1.2rem' }}>→</span>
              </a>
            </div>
          </div>

          {/* CARDS GRID */}
          <div className="savings-products__grid">
            {products.map((product, index) => (
              <ProductCard 
                key={index}
                title={product.title}
                description={product.description}
                icon={product.icon}
              />
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};

export default SavingsProducts;
