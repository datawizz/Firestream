import React from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

const Layout = ({ children }) => {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLoginClick = () => {
    navigate('/login');
  };

  return (
    <div className="app-layout">
      <header className="header">
        <nav className="nav container">
          <div className="nav-left">
            <Link to="/" className="logo">Example Shop</Link>
            <ul className="nav-links">
              <li><Link to="/">Home</Link></li>
              {user && (
                <>
                  <li><Link to="/dashboard">Dashboard</Link></li>
                  <li><Link to="/products">Products</Link></li>
                </>
              )}
            </ul>
          </div>
          <div className="nav-right">
            {user ? (
              <div className="user-menu">
                <span>Welcome, {user.username}</span>
                <button onClick={logout} className="logout-button">Logout</button>
              </div>
            ) : (
              <button onClick={handleLoginClick} className="login-button">
                Login
              </button>
            )}
          </div>
        </nav>
      </header>
      
      <main className="main-content">
        <div className="container">
          {children}
        </div>
      </main>
      
      <footer className="footer">
        <div className="container">
          <p>&copy; 2024 Example Shop. All rights reserved.</p>
        </div>
      </footer>
    </div>
  );
};

export default Layout;