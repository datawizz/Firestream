import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { AuthProvider } from './context/AuthContext';
import Layout from './components/Layout';
import LoginForm from './components/LoginForm';
import Dashboard from './components/Dashboard';
import ProductList from './components/ProductList';

function App() {
  return (
    <Router>
      <AuthProvider>
        <Layout>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/login" element={<LoginForm />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/products" element={<ProductList />} />
          </Routes>
        </Layout>
      </AuthProvider>
    </Router>
  );
}

function Home() {
  return (
    <div className="home">
      <h1>Welcome to Example Shop</h1>
      <p>Your premier destination for quality products</p>
    </div>
  );
}

export default App;