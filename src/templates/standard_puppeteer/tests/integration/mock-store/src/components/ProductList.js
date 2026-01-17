import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import ProductItem from './ProductItem';
import Pagination from './Pagination';
import { mockProducts } from '../data/mockProducts';

const PRODUCTS_PER_PAGE = 20;

const ProductList = () => {
  const [currentPage, setCurrentPage] = useState(1);
  const [loading, setLoading] = useState(true);
  const { user } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    // Redirect to login if not authenticated
    if (!user) {
      navigate('/login');
      return;
    }
    
    // Simulate loading delay
    setTimeout(() => {
      setLoading(false);
    }, 500);
  }, [user, navigate]);

  if (!user) {
    return null;
  }

  if (loading) {
    return <div className="loading">Loading products...</div>;
  }

  // Calculate pagination
  const totalPages = Math.ceil(mockProducts.length / PRODUCTS_PER_PAGE);
  const startIndex = (currentPage - 1) * PRODUCTS_PER_PAGE;
  const endIndex = startIndex + PRODUCTS_PER_PAGE;
  const currentProducts = mockProducts.slice(startIndex, endIndex);

  const handlePageChange = (newPage) => {
    setCurrentPage(newPage);
    // Scroll to top when page changes
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  return (
    <div className="products-page">
      <h1>Products</h1>
      <p>Showing {startIndex + 1}-{Math.min(endIndex, mockProducts.length)} of {mockProducts.length} products</p>
      
      <div className="product-grid">
        {currentProducts.map(product => (
          <ProductItem key={product.id} product={product} />
        ))}
      </div>
      
      <Pagination 
        currentPage={currentPage}
        totalPages={totalPages}
        onPageChange={handlePageChange}
      />
    </div>
  );
};

export default ProductList;