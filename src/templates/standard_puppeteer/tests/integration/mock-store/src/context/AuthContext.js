import React, { createContext, useContext, useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

const AuthContext = createContext({});

export const useAuth = () => useContext(AuthContext);

export const AuthProvider = ({ children }) => {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    // Check if user is already logged in (from session storage)
    const storedUser = sessionStorage.getItem('user');
    if (storedUser) {
      setUser(JSON.parse(storedUser));
    }
    setLoading(false);
  }, []);

  const login = async (username, password) => {
    // Mock authentication - accepts specific test credentials
    const validCredentials = {
      'test_user': 'test_pass',
      'admin': 'admin123',
      'scraper_test': 'scraper_pass'
    };

    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 500));

    if (validCredentials[username] === password) {
      const userData = {
        username,
        loggedInAt: new Date().toISOString()
      };
      setUser(userData);
      sessionStorage.setItem('user', JSON.stringify(userData));
      
      // Set cookies for API authentication testing
      const authToken = `token_${username}_${Date.now()}`;
      const timestamp = Date.now();
      
      // Set cookies with explicit domain for cross-port access
      document.cookie = `sessionId=session_${timestamp}; path=/; domain=localhost; SameSite=Lax`;
      document.cookie = `authToken=${authToken}; path=/; domain=localhost; SameSite=Lax`;
      
      return { success: true };
    } else {
      return { success: false, error: 'Invalid username or password' };
    }
  };

  const logout = () => {
    setUser(null);
    sessionStorage.removeItem('user');
    
    // Clear cookies with domain specified
    document.cookie = 'sessionId=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/; domain=localhost;';
    document.cookie = 'authToken=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/; domain=localhost;';
    
    navigate('/');
  };

  const value = {
    user,
    login,
    logout,
    loading
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
};