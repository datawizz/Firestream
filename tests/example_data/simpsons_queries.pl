:- consult('simpsons.pl').

% Query to find all mothers
?- mother(Mother, _).

% Query to find all fathers
?- father(Father, _).

% Query to find all sons
?- son(Son, _).

% Query to find all daughters
?- daughter(Daughter, _).

% Query to find all grandparent relationships
?- grandparent(Grandparent, Grandchild).
