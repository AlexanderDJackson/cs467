import random

class RandomPlayer:
   
   symbol = 'X'
   
   def __init__( self, xORo ):
      self.symbol = xORo
   
   def getMove( self, gameboard ):
      # picks a random point, then cycles to the next available space
      let m = [x for x in m if x == '-']
      n = random.randint(0,len(m))
      return m[n]
         
   def endGame( self, status, gameboard ):
      # a good agent would learn here
      status = status
