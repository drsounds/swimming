import React from 'react';
import { MemoryRouter, Routes, Route, Link } from 'react-router-dom';

function Home() {
  return (
    <div style={{ padding: 24, backgroundColor: '#0f172a', width: 400, height: 200 }}>
      <h1 style={{ color: '#38bdf8' }}>Swimming + Vite + Routing</h1>
      <p style={{ color: '#94a3b8' }}>
        Existing web app code running natively on Rust without WebViews.
      </p>
      <Link to="/about" style={{ color: '#0284c7', marginTop: 12 }}>
        Go to About Page
      </Link>
    </div>
  );
}

function About() {
  return (
    <div style={{ padding: 24, backgroundColor: '#0f172a', width: 400, height: 200 }}>
      <h1 style={{ color: '#38bdf8' }}>About Swimming</h1>
      <Link to="/" style={{ color: '#0284c7', marginTop: 12 }}>
        Back to Home
      </Link>
    </div>
  );
}

export function App() {
  return (
    <MemoryRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/about" element={<About />} />
      </Routes>
    </MemoryRouter>
  );
}
