"use client";

import React, { useState } from "react";
import "./Newsletter.css";

const Newsletter: React.FC = () => {
  const [email, setEmail] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (email) {
      console.log("Newsletter signup:", email);
      // Here you would typically send the email to your backend
      setEmail("");
    }
  };

  return (
    <section className="newsletter">
      <div className="newsletter__container">
        <div className="newsletter__text">
          <h2 className="newsletter__title">
            Want to receive any updates or news?
          </h2>
          <p className="newsletter__subtitle">Sign up for our Newsletter</p>
        </div>

        <form className="newsletter__form" onSubmit={handleSubmit}>
          <div className="newsletter__input-wrapper">
            <input
              type="email"
              className="newsletter__input"
              placeholder="Enter your email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </div>
          <button type="submit" className="newsletter__button">
            Submit
          </button>
        </form>
      </div>
    </section>
  );
};

export default Newsletter;
