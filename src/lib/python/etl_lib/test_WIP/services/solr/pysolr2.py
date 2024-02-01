# If on Python 2.X
from __future__ import print_function

import pysolr
import os

os.environ["DEBUG_PYSOLR"] = "true"

zookeeper = pysolr.ZooKeeper("solr-zookeeper.default.svc.cluster.local:2181")
solr = pysolr.SolrCloud(zookeeper, "my-collection", auth=None)


# Create a client instance. The timeout and authentication options are not required.
#solr = pysolr.Solr('http://solr.default.svc.cluster.local:8983/solr/', always_commit=True)

# Note that auto_commit defaults to False for performance. You can set
# `auto_commit=True` to have commands always update the index immediately, make
# an update call with `commit=True`, or use Solr's `autoCommit` / `commitWithin`
# to have your data be committed following a particular policy.

# Do a health check.
solr.ping()

# # How you'd index data.
# solr.add([
#     {
#         "id": "doc_1",
#         "title": "A test document",
#     },
#     {
#         "id": "doc_2",
#         "title": "The Banana: Tasty or Dangerous?",
#         "_doc": [
#             { "id": "child_doc_1", "title": "peel" },
#             { "id": "child_doc_2", "title": "seed" },
#         ]
#     },
# ])

# # You can index a parent/child document relationship by
# # associating a list of child documents with the special key '_doc'. This
# # is helpful for queries that join together conditions on children and parent
# # documents.

# # Later, searching is easy. In the simple case, just a plain Lucene-style
# # query is fine.
# results = solr.search('bananas')

# # The ``Results`` object stores total results found, by default the top
# # ten most relevant results and any additional data like
# # facets/highlighting/spelling/etc.
# print("Saw {0} result(s).".format(len(results)))

# # Just loop over it to access the results.
# for result in results:
#     print("The title is '{0}'.".format(result['title']))

# # For a more advanced query, say involving highlighting, you can pass
# # additional options to Solr.
# results = solr.search('bananas', **{
#     'hl': 'true',
#     'hl.fragsize': 10,
# })

# # Traverse a cursor using its iterator:
# for doc in solr.search('*:*',fl='id',sort='id ASC',cursorMark='*'):
#     print(doc['id'])

# # You can also perform More Like This searches, if your Solr is configured
# # correctly.
# similar = solr.more_like_this(q='id:doc_2', mltfl='text')

# # Finally, you can delete either individual documents,
# solr.delete(id='doc_1')

# # also in batches...
# solr.delete(id=['doc_1', 'doc_2'])

# # ...or all documents.
# solr.delete(q='*:*')