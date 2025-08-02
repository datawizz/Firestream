import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

const Dashboard = () => {
  const { user } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    // Redirect to login if not authenticated
    if (!user) {
      navigate('/login');
    }
  }, [user, navigate]);

  if (!user) {
    return null;
  }

  // Mock statistics
  const stats = {
    totalProducts: 100,
    inStock: 80,
    outOfStock: 20,
    categories: 5
  };

  return (
    <div className="dashboard">
      <h1>Dashboard</h1>
      <p>Welcome back, {user.username}!</p>
      
      <div className="dashboard-grid">
        <div className="dashboard-card">
          <h3>Total Products</h3>
          <div className="value">{stats.totalProducts}</div>
        </div>
        
        <div className="dashboard-card">
          <h3>In Stock</h3>
          <div className="value">{stats.inStock}</div>
        </div>
        
        <div className="dashboard-card">
          <h3>Out of Stock</h3>
          <div className="value">{stats.outOfStock}</div>
        </div>
        
        <div className="dashboard-card">
          <h3>Categories</h3>
          <div className="value">{stats.categories}</div>
        </div>
      </div>
      
      <div style={{ marginTop: '40px' }}>
        <button 
          onClick={() => navigate('/products')}
          className="login-button"
          style={{ padding: '12px 24px', fontSize: '16px' }}
        >
          View All Products
        </button>
      </div>
    </div>
  );
};

export default Dashboard;