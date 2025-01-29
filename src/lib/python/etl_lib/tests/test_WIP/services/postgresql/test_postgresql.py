from sqlalchemy import create_engine, Column, Integer, String, DateTime, Text, ForeignKey
from sqlalchemy.engine import URL
from sqlalchemy.orm import declarative_base, relationship, sessionmaker
from datetime import datetime
import pytest, os
from sqlalchemy.exc import IntegrityError

Base = declarative_base()

# Read environment variables
POSTGRES_USER = os.environ['POSTGRES_USER']
POSTGRES_PASSWORD = os.environ['POSTGRES_PASSWORD']
POSTGRES_DEFAULT_DB = os.environ['POSTGRES_DEFAULT_DB']
POSTGRES_URL = os.environ['POSTGRES_URL']
POSTGRES_PORT = os.environ['POSTGRES_PORT']



connection_string = f"postgresql://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{POSTGRES_URL}:{POSTGRES_PORT}/{POSTGRES_DEFAULT_DB}"



Base = declarative_base()


class Author(Base):
    __tablename__ = 'authors'

    id = Column(Integer(), primary_key=True)
    firstname = Column(String(100))
    lastname = Column(String(100))
    email = Column(String(255), nullable=False)
    joined = Column(DateTime(), default=datetime.now)

    articles = relationship('Article', backref='author')


class Article(Base):
    __tablename__ = 'articles'

    id = Column(Integer(), primary_key=True)
    slug = Column(String(100), nullable=False)
    title = Column(String(100), nullable=False)
    created_on = Column(DateTime(), default=datetime.now)
    updated_on = Column(DateTime(), default=datetime.now, onupdate=datetime.now)
    content = Column(Text)
    author_id = Column(Integer(), ForeignKey('authors.id'))




# ========= Tests ==========

class TestBlog:
    def setup_class(self):
        Base.metadata.create_all(engine)
        self.session = Session()
        self.valid_author = Author(
            firstname="Ezzeddin",
            lastname="Aybak",
            email="aybak_email@gmail.com"
        )

    def teardown_class(self):
        self.session.rollback()
        self.session.close()

    def test_author_valid(self):   
        self.session.add(self.valid_author)
        self.session.commit()
        aybak = self.session.query(Author).filter_by(lastname="Aybak").first()
        print(aybak.__dict__)
        assert aybak.firstname == "Ezzeddin"
        assert aybak.lastname != "Abdullah"
        assert aybak.email == "aybak_email@gmail.com"


    @pytest.mark.xfail(raises=IntegrityError)
    def test_author_no_email(self):
        author = Author(
            firstname="James",
            lastname="Clear"
        )
        self.session.add(author)
        try:
            self.session.commit()
        except IntegrityError:
            self.session.rollback()

    def test_article_valid(self):
        valid_article = Article(
            slug="sample-slug",
            title="Title of the Valid Article",
            content="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
            author=self.valid_author
            )
        self.session.add(valid_article)
        self.session.commit()
        sample_article = self.session.query(Article).filter_by(slug="sample-slug").first()
        assert sample_article.title == "Title of the Valid Article"
        assert len(sample_article.content.split(" ")) > 50


if __name__ == "__main__":
    engine = create_engine(connection_string)
    Session = sessionmaker(bind=engine)
    _class = TestBlog()
    _class.setup_class()
    _class.test_author_valid()
    _class.test_article_valid()
    _class.test_author_no_email()
    _class.teardown_class()
    
    print("Everything passed")
