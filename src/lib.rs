use std::rc::Rc;
use std::cell::RefCell;
use std::boxed;

pub mod ui;

pub mod manager;

pub mod typesetter;

pub mod parser;

pub mod analyzer;

pub type Callbacks<T> = Rc<RefCell<Vec<boxed::Box<dyn Fn(T) + 'static>>>>;

pub type ValuedCallbacks<A, R> = Rc<RefCell<Vec<boxed::Box<dyn Fn(A)->R + 'static>>>>;

pub trait React<S> {

    fn react(&self, source : &S);

}

/*
\documentclass{...}
<<Preamble>>
\begin{document}
<<Document>>
\end{document}


\title{How to Structure a LaTeX Document}
\author{Andrew Roberts}
\date{December 2004}
\maketitle

\begin{abstract}
Your abstract goes here...
...
\end{abstract}
\renewcommand{\abstractname}{Executive Summary}

\chapter
\section
\subsection
\clearpage
\tableofcontents
\listoffigures
\listoftables

(For books)
\frontmatter

\mainmatter
\chapter{First chapter}
% ...

\appendix
\chapter{First Appendix}

\backmatter
\chapter{Last note}

\bibliography


-- Formatting
\linespread{factor}

\vfill

\clearpage

\parindent

\begin{list_type}
\item The first item
\item The second item
\item The third etc \ldots
\end{list_type}

\topmargin
\includegraphics[width=.3\linewidth]{example-image}
\footnotemark
\usepackage{hyperref}
\hyperref[label_name]{''link text''}

\footnotetext{This is my footnote!}
*/


