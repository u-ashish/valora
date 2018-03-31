module Traits.Position
  ( Translate(..)
  , Centered(..)
  , Place(..)
  ) where

import Coords

class Centered c where
  center :: c -> Point

class Translate t where
  translate :: Point -> t -> t

class Place p where
  place :: Point -> p -> p

instance (Translate a, Centered a) => Place a where
  place dest a = translate (dest - center a) a
