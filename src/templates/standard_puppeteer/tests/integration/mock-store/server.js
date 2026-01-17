const express = require('express');
const cors = require('cors');
const cookieParser = require('cookie-parser');
const session = require('express-session');
const { mockProducts } = require('./src/data/mockProducts');

const app = express();
const PORT = process.env.API_PORT || 3001;

// Middleware
app.use(cors({
  origin: (origin, callback) => {
    // Allow all localhost origins for testing
    if (!origin || origin.includes('localhost') || origin.includes('127.0.0.1')) {
      callback(null, true);
    } else {
      callback(new Error('Not allowed by CORS'));
    }
  },
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization', 'Cookie', 'X-Requested-With'],
  exposedHeaders: ['Set-Cookie'],
  preflightContinue: false,
  optionsSuccessStatus: 204
}));
app.use(express.json());
app.use(cookieParser());

// Session configuration
app.use(session({
  secret: 'mock-store-secret-key',
  resave: false,
  saveUninitialized: false,
  cookie: {
    secure: false, // Set to true in production with HTTPS
    httpOnly: true,
    maxAge: 24 * 60 * 60 * 1000, // 24 hours
    sameSite: 'lax',
    domain: 'localhost', // Allow cross-port access
    path: '/'
  },
  name: 'connect.sid' // Use default session name to avoid conflicts
}));

// Valid credentials for API authentication
const validCredentials = {
  'test_user': 'test_pass',
  'admin': 'admin123',
  'scraper_test': 'scraper_pass'
};

// Authentication middleware
const requireAuth = (req, res, next) => {
  // Debug logging
  console.log(`[AUTH] ${req.method} ${req.path} - Headers:`, {
    origin: req.get('origin'),
    referer: req.get('referer'),
    cookie: req.get('cookie'),
    host: req.get('host')
  });
  console.log('[AUTH] Parsed cookies:', req.cookies);
  console.log('[AUTH] Session data:', {
    sessionId: req.sessionID,
    authenticated: req.session?.authenticated,
    username: req.session?.username
  });
  
  // Check session-based auth (from API login)
  if (req.session && req.session.authenticated) {
    console.log('[AUTH] Authenticated via session');
    return next();
  }
  
  // Check cookie-based auth (from frontend login)
  const authToken = req.cookies.authToken;
  const sessionId = req.cookies.sessionId;
  
  // Check Origin header for same-site requests (browser-based API calls)
  const origin = req.get('origin');
  const referer = req.get('referer');
  const isSameSite = (origin && origin.includes('localhost')) || 
                     (referer && referer.includes('localhost'));
  
  console.log('[AUTH] Same-site:', isSameSite, 'authToken:', !!authToken, 'sessionId:', !!sessionId);
  
  // More flexible cookie validation for cross-port requests
  if (isSameSite && (authToken || sessionId)) {
    // Either cookie is sufficient for same-site requests
    if (authToken && authToken.startsWith('token_')) {
      console.log('[AUTH] Authenticated via authToken (same-site)');
      return next();
    }
    if (sessionId && sessionId.startsWith('session_')) {
      console.log('[AUTH] Authenticated via sessionId (same-site)');
      return next();
    }
  }
  
  // Strict validation for non-same-site requests
  if (!isSameSite && authToken && sessionId && 
      authToken.startsWith('token_') && sessionId.startsWith('session_')) {
    console.log('[AUTH] Authenticated via both cookies (non-same-site)');
    return next();
  }
  
  console.log('[AUTH] Authentication failed');
  return res.status(401).json({ 
    error: 'Authentication required',
    message: 'Please log in to access this resource'
  });
};

// API Routes

// Authentication endpoint
app.post('/api/auth/login', (req, res) => {
  const { username, password } = req.body;

  // Simulate API delay
  setTimeout(() => {
    if (validCredentials[username] === password) {
      req.session.authenticated = true;
      req.session.username = username;
      req.session.loginTime = new Date().toISOString();
      
      // Set additional auth cookie for testing
      res.cookie('authToken', `token_${username}_${Date.now()}`, {
        httpOnly: false, // Allow client-side access for testing
        secure: false,
        maxAge: 24 * 60 * 60 * 1000,
        sameSite: 'lax',
        domain: 'localhost', // Allow cross-port access
        path: '/'
      });

      res.json({
        success: true,
        user: {
          username,
          loginTime: req.session.loginTime
        },
        message: 'Login successful'
      });
    } else {
      res.status(401).json({
        success: false,
        error: 'Invalid credentials',
        message: 'Username or password is incorrect'
      });
    }
  }, 500);
});

// Logout endpoint
app.post('/api/auth/logout', (req, res) => {
  req.session.destroy((err) => {
    if (err) {
      return res.status(500).json({ error: 'Could not log out' });
    }
    res.clearCookie('connect.sid', { domain: 'localhost', path: '/' });
    res.clearCookie('authToken', { domain: 'localhost', path: '/' });
    res.json({ success: true, message: 'Logged out successfully' });
  });
});

// Check authentication status
app.get('/api/auth/status', (req, res) => {
  console.log('[AUTH STATUS] Checking authentication...');
  console.log('[AUTH STATUS] Cookies:', req.cookies);
  console.log('[AUTH STATUS] Session:', {
    id: req.sessionID,
    authenticated: req.session?.authenticated
  });
  
  // Check session-based auth (from API login)
  if (req.session && req.session.authenticated) {
    res.json({
      authenticated: true,
      user: {
        username: req.session.username,
        loginTime: req.session.loginTime
      },
      authMethod: 'session'
    });
    return;
  }
  
  // Check cookie-based auth (from frontend login)
  const authToken = req.cookies.authToken;
  const sessionId = req.cookies.sessionId;
  
  // Check Origin header for same-site requests
  const origin = req.get('origin');
  const referer = req.get('referer');
  const isSameSite = (origin && origin.includes('localhost')) || 
                     (referer && referer.includes('localhost'));
  
  console.log('[AUTH STATUS] Same-site:', isSameSite, 'authToken:', !!authToken, 'sessionId:', !!sessionId);
  
  // More flexible validation for cross-port requests
  if (isSameSite && (authToken || sessionId)) {
    if (authToken && authToken.startsWith('token_')) {
      // Extract username from token (format: token_username_timestamp)
      const tokenParts = authToken.split('_');
      const username = tokenParts.length >= 3 ? tokenParts[1] : 'unknown';
      
      res.json({
        authenticated: true,
        user: {
          username: username,
          loginTime: new Date().toISOString(),
          authType: 'cookie'
        },
        authMethod: 'cookie-token'
      });
      return;
    }
    
    if (sessionId && sessionId.startsWith('session_')) {
      res.json({
        authenticated: true,
        user: {
          username: 'unknown',
          loginTime: new Date().toISOString(),
          authType: 'cookie'
        },
        authMethod: 'cookie-session'
      });
      return;
    }
  }
  
  res.json({ 
    authenticated: false,
    debug: {
      hasAuthToken: !!authToken,
      hasSessionId: !!sessionId,
      isSameSite: isSameSite
    }
  });
});

// Protected API endpoints

// Get all products (with pagination)
app.get('/api/products', requireAuth, (req, res) => {
  const page = parseInt(req.query.page) || 1;
  const limit = parseInt(req.query.limit) || 20;
  const category = req.query.category;
  const inStock = req.query.inStock;

  let filteredProducts = [...mockProducts];

  // Apply filters
  if (category) {
    filteredProducts = filteredProducts.filter(p => 
      p.category.toLowerCase() === category.toLowerCase()
    );
  }

  if (inStock !== undefined) {
    const stockFilter = inStock === 'true';
    filteredProducts = filteredProducts.filter(p => p.inStock === stockFilter);
  }

  // Pagination
  const startIndex = (page - 1) * limit;
  const endIndex = startIndex + limit;
  const paginatedProducts = filteredProducts.slice(startIndex, endIndex);

  res.json({
    products: paginatedProducts,
    pagination: {
      page,
      limit,
      total: filteredProducts.length,
      pages: Math.ceil(filteredProducts.length / limit)
    },
    filters: {
      category: category || null,
      inStock: inStock || null
    }
  });
});

// Get single product
app.get('/api/products/:id', requireAuth, (req, res) => {
  const product = mockProducts.find(p => p.id === req.params.id);
  
  if (product) {
    res.json({ product });
  } else {
    res.status(404).json({ 
      error: 'Product not found',
      id: req.params.id
    });
  }
});

// Get product categories
app.get('/api/categories', requireAuth, (req, res) => {
  const categories = [...new Set(mockProducts.map(p => p.category))];
  res.json({ categories });
});

// Get product stats
app.get('/api/stats', requireAuth, (req, res) => {
  const totalProducts = mockProducts.length;
  const inStockCount = mockProducts.filter(p => p.inStock).length;
  const avgPrice = mockProducts.reduce((sum, p) => sum + p.price, 0) / totalProducts;
  
  res.json({
    total: totalProducts,
    inStock: inStockCount,
    outOfStock: totalProducts - inStockCount,
    averagePrice: parseFloat(avgPrice.toFixed(2)),
    categories: [...new Set(mockProducts.map(p => p.category))].length
  });
});

// Health check endpoint (public)
app.get('/api/health', (req, res) => {
  res.json({ 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    authenticated: !!(req.session && req.session.authenticated)
  });
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error('API Error:', err);
  res.status(500).json({ 
    error: 'Internal server error',
    message: err.message 
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({ 
    error: 'Endpoint not found',
    path: req.path 
  });
});

app.listen(PORT, () => {
  console.log(`Mock Store API Server running on http://localhost:${PORT}`);
  console.log(`Available endpoints:`);
  console.log(`  POST /api/auth/login - Authenticate user`);
  console.log(`  POST /api/auth/logout - Logout user`);
  console.log(`  GET  /api/auth/status - Check auth status`);
  console.log(`  GET  /api/products - Get products (protected)`);
  console.log(`  GET  /api/products/:id - Get single product (protected)`);
  console.log(`  GET  /api/categories - Get categories (protected)`);
  console.log(`  GET  /api/stats - Get statistics (protected)`);
  console.log(`  GET  /api/health - Health check`);
});

module.exports = app;